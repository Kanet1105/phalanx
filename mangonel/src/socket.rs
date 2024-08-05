use std::{
    ffi::{CString, NulError},
    io,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    sync::Arc,
};

use libc::XDP_USE_NEED_WAKEUP;
use mangonel_libxdp_sys::{
    xsk_ring_cons, xsk_ring_cons__peek, xsk_ring_cons__release, xsk_ring_cons__rx_desc,
    xsk_ring_prod, xsk_ring_prod__reserve, xsk_socket, xsk_socket__create, xsk_socket__delete,
    xsk_socket_config, xsk_socket_config__bindgen_ty_1, XDP_SHARED_UMEM,
    XSK_LIBXDP_FLAGS__INHIBIT_PROG_LOAD,
};

use crate::umem::{Umem, UmemError};

pub struct Socket {
    inner: Arc<SocketInner>,
}

struct SocketInner {
    umem: Umem,
    socket: NonNull<xsk_socket>,
}

impl Drop for SocketInner {
    fn drop(&mut self) {
        unsafe { xsk_socket__delete(self.socket.as_ptr()) }
    }
}

impl Clone for Socket {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Socket {
    pub fn new(
        packet_size: usize,
        buffer_length: usize,
        queue_id: u32,
        rx_ring_size: u32,
        tx_ring_size: u32,
        interface_name: impl AsRef<str>,
    ) -> Result<(Receiver, Sender), SocketError> {
        let umem = Umem::new(packet_size, buffer_length, rx_ring_size, tx_ring_size).unwrap();

        let socket_config = xsk_socket_config {
            rx_size: rx_ring_size,
            tx_size: tx_ring_size,
            __bindgen_anon_1: xsk_socket_config__bindgen_ty_1 { libbpf_flags: 0 },
            xdp_flags: XSK_LIBXDP_FLAGS__INHIBIT_PROG_LOAD | XDP_SHARED_UMEM,
            bind_flags: XDP_USE_NEED_WAKEUP,
        };

        let mut socket: *mut xsk_socket = ptr::null_mut();

        let interface_name =
            CString::new(interface_name.as_ref()).map_err(SocketError::InterfaceName)?;

        let mut rx_ring = MaybeUninit::<xsk_ring_cons>::zeroed();
        let mut tx_ring = MaybeUninit::<xsk_ring_prod>::zeroed();

        let value = unsafe {
            xsk_socket__create(
                &mut socket,
                interface_name.as_ptr(),
                queue_id,
                umem.as_ptr(),
                rx_ring.as_mut_ptr(),
                tx_ring.as_mut_ptr(),
                &socket_config,
            )
        };

        if value.is_negative() {
            return Err(SocketError::Initialize(io::Error::from_raw_os_error(
                -value,
            )));
        }

        let rx_ring = unsafe { rx_ring.assume_init() };
        let tx_ring = unsafe { tx_ring.assume_init() };

        let socket = Socket {
            inner: Arc::new(SocketInner {
                umem,
                socket: NonNull::new(socket).ok_or(SocketError::SocketIsNull)?,
            }),
        };

        let receiver = Receiver::new(rx_ring, socket.clone());
        let sender = Sender::new(tx_ring, socket.clone());

        Ok((receiver, sender))
    }

    pub fn umem(&self) -> &Umem {
        &self.inner.umem
    }
}

#[derive(Debug)]
pub struct Packet<'a> {
    pub address: u64,
    pub length: u32,
    pub data: &'a mut [u8],
}

pub struct Receiver {
    rx_ring: xsk_ring_cons,
    socket: Socket,
}

unsafe impl Send for Receiver {}

impl Receiver {
    pub fn new(rx_ring: xsk_ring_cons, socket: Socket) -> Self {
        Self { socket, rx_ring }
    }

    pub fn receive(&mut self, buffer: &mut Vec<Packet>) -> u32 {
        let mut index: u32 = 0;

        let received =
            unsafe { xsk_ring_cons__peek(self.as_ptr(), buffer.len() as u32, &mut index) };

        if received == 0 {
            return received;
        }

        for _ in 0..received {
            unsafe {
                let descriptor = xsk_ring_cons__rx_desc(self.as_ptr(), index);
                let address = (*descriptor).addr;
                let length = (*descriptor).len;

                let packet = Packet {
                    address,
                    length,
                    data: std::slice::from_raw_parts_mut(
                        self.socket.umem().offset(address as isize) as *mut u8,
                        self.socket.umem().packet_size(),
                    ),
                };

                buffer.push(packet);
                index += 1;
            }
        }

        unsafe { xsk_ring_cons__release(self.as_ptr(), received) }

        received
    }

    fn as_ptr(&self) -> *mut xsk_ring_cons {
        self.rx_ring.ring as *mut xsk_ring_cons
    }
}

pub struct Sender {
    tx_ring: xsk_ring_prod,
    socket: Socket,
}

unsafe impl Send for Sender {}

impl Sender {
    pub fn new(tx_ring: xsk_ring_prod, socket: Socket) -> Self {
        Self { tx_ring, socket }
    }

    pub fn send(&mut self, buffer: &mut Vec<u8>) {
        let mut index: u32 = 0;
        let batch_size = buffer.len() as u32;

        let available = unsafe { xsk_ring_prod__reserve(self.as_ptr(), batch_size, &mut index) };
        for _ in 0..available {
            // let packet
        }
    }

    fn as_ptr(&self) -> *mut xsk_ring_prod {
        self.tx_ring.ring as *mut xsk_ring_prod
    }
}

#[derive(Debug)]
pub enum SocketError {
    Umem(UmemError),
    InterfaceName(NulError),
    Initialize(std::io::Error),
    SocketIsNull,
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SocketError {}
