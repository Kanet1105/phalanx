use std::{
    collections::VecDeque,
    ffi::{CString, NulError},
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use arraydeque::{ArrayDeque, Wrapping};
use libc::{poll, pollfd, sendto, MSG_DONTWAIT, POLLIN, XDP_USE_NEED_WAKEUP};
use mangonel_libxdp_sys::{
    xdp_desc, xsk_socket, xsk_socket__create, xsk_socket__delete, xsk_socket__fd,
    xsk_socket_config, xsk_socket_config__bindgen_ty_1, XDP_SHARED_UMEM,
    XSK_RING_CONS__DEFAULT_NUM_DESCS, XSK_RING_PROD__DEFAULT_NUM_DESCS,
    XSK_UMEM__DEFAULT_FRAME_HEADROOM, XSK_UMEM__DEFAULT_FRAME_SIZE,
};

use crate::{
    mmap::{Mmap, MmapError},
    packet::Frame,
    ring::{RingError, RxRing, TxRing},
    umem::{Umem, UmemError},
};

#[derive(Debug)]
pub struct SocketBuilder {
    pub frame_size: u32,
    pub frame_headroom_size: u32,
    pub descriptor_count: u32,
    pub completion_ring_size: u32,
    pub fill_ring_size: u32,
    pub rx_ring_size: u32,
    pub tx_ring_size: u32,
}

impl Default for SocketBuilder {
    fn default() -> Self {
        Self {
            frame_size: XSK_UMEM__DEFAULT_FRAME_SIZE,
            frame_headroom_size: XSK_UMEM__DEFAULT_FRAME_HEADROOM,
            descriptor_count: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            completion_ring_size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            fill_ring_size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
            rx_ring_size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            tx_ring_size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
        }
    }
}

impl SocketBuilder {
    // pub fn build(
    //     self,
    //     interface_name: impl AsRef<str>,
    //     queue_id: u32,
    // ) -> Result<(RxSocket, TxSocket), SocketError> { let mmap = Mmap::initialize(
    //   self.frame_size + self.frame_headroom, self.descriptor_count, false, )?;

    //     let umem = Umem::initialize(
    //         self.completion_ring_size,
    //         self.fill_ring_size,
    //         self.frame_size,
    //         self.frame_headroom,
    //         mmap,
    //     )
    //     .map_err(SocketError::Umem)?;

    //     Socket::initialize(
    //         self.rx_ring_size,
    //         self.tx_ring_size,
    //         interface_name,
    //         queue_id,
    //         umem,
    //     )
    // }
    pub fn build(
        self,
        interface_name: impl AsRef<str>,
        queue_id: u32,
        umem: &Umem,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        Socket::initialize(
            self.rx_ring_size,
            self.tx_ring_size,
            interface_name,
            queue_id,
            umem,
        )
    }
}

pub struct Socket {
    inner: Arc<SocketInner>,
}

struct SocketInner(NonNull<xsk_socket>);

impl Drop for SocketInner {
    fn drop(&mut self) {
        unsafe { xsk_socket__delete(self.0.as_ptr()) }
    }
}

unsafe impl Send for Socket {}

unsafe impl Sync for Socket {}

impl Clone for Socket {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Socket {
    pub fn initialize(
        rx_ring_size: u32,
        tx_ring_size: u32,
        interface_name: impl AsRef<str>,
        queue_id: u32,
        umem: &Umem,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        let mut rx_ring = RxRing::uninitialized(rx_ring_size)?;
        let mut tx_ring = TxRing::uninitialized(tx_ring_size)?;
        let interface_name =
            CString::new(interface_name.as_ref()).map_err(SocketError::InterfaceName)?;
        let socket_config = xsk_socket_config {
            rx_size: rx_ring_size,
            tx_size: tx_ring_size,
            __bindgen_anon_1: xsk_socket_config__bindgen_ty_1 { libxdp_flags: 0 },
            xdp_flags: XDP_SHARED_UMEM,
            bind_flags: XDP_USE_NEED_WAKEUP,
        };
        let mut socket = null_mut();

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
            return Err(SocketError::Initialize(std::io::Error::from_raw_os_error(
                -value,
            )));
        }

        let inner = SocketInner(NonNull::new(socket).unwrap());
        let socket = Self {
            inner: Arc::new(inner),
        };
        let rx_socket = RxSocket::new(socket.clone(), rx_ring.initialize()?, umem.clone());
        let tx_socket = TxSocket::new(socket.clone(), tx_ring.initialize()?, umem.clone());

        Ok((rx_socket, tx_socket))
    }

    pub fn as_ptr(&self) -> *mut xsk_socket {
        self.inner.0.as_ptr()
    }

    pub fn socket_fd(&self) -> i32 {
        unsafe { xsk_socket__fd(self.as_ptr()) }
    }

    pub fn poll_fd(&self) {
        let mut poll_fd_struct = pollfd {
            fd: self.socket_fd(),
            events: POLLIN,
            revents: 0,
        };

        unsafe { poll(&mut poll_fd_struct, 1, 0) };
    }
}

pub struct RxSocket {
    socket: Socket,
    rx_ring: RxRing,
    umem: Umem,
}

impl RxSocket {
    pub fn new(socket: Socket, rx_ring: RxRing, umem: Umem) -> Self {
        Self {
            socket,
            rx_ring,
            umem,
        }
    }

    pub fn as_ptr(&self) -> *mut xsk_socket {
        self.socket.as_ptr()
    }

    pub fn socket_fd(&self) -> i32 {
        unsafe { xsk_socket__fd(self.as_ptr()) }
    }

    pub fn rx_ring(&self) -> &RxRing {
        &self.rx_ring
    }

    pub fn rx_burst(&mut self, buffer: &mut ArrayDeque<Frame, 128, Wrapping>, mmap: &Mmap) -> u32 {
        let burst_size = buffer.capacity() as u32;
        let mut index: u32 = 0;

        let received = self.rx_ring.peek(burst_size, &mut index);
        if received == 0 {
            if self.umem.fill_ring().needs_wakeup() {
                self.socket.poll_fd();
            }

            return received;
        }

        for _ in 0..received {
            let frame = unsafe {
                let descriptor: *const xdp_desc = self.rx_ring.rx_descriptor(index);
                let address = (*descriptor).addr;
                let length = (*descriptor).len;
                let frame_address = mmap.offset(address as isize);

                Frame {
                    address,
                    length,
                    data: std::slice::from_raw_parts_mut(
                        frame_address as *mut u8,
                        mmap.frame_size().try_into().unwrap(),
                    ),
                }
            };

            buffer.push_back(frame);
            index += 1;
        }

        self.rx_ring.release(received);

        received
    }
}

pub struct TxSocket {
    socket: Socket,
    tx_ring: TxRing,
    umem: Umem,
}

impl TxSocket {
    pub fn new(socket: Socket, tx_ring: TxRing, umem: Umem) -> Self {
        Self {
            socket,
            tx_ring,
            umem,
        }
    }

    pub fn as_ptr(&self) -> *mut xsk_socket {
        self.socket.as_ptr()
    }

    pub fn socket_fd(&self) -> i32 {
        unsafe { xsk_socket__fd(self.as_ptr()) }
    }

    pub fn tx_burst(&mut self, buffer: &mut VecDeque<Frame>) -> u32 {
        let mut index: u32 = 0;

        unsafe { sendto(self.socket_fd(), null_mut(), 0, MSG_DONTWAIT, null_mut(), 0) };

        let completed = self.umem.completion_ring().peek(&mut index);
        if completed > 0 {
            for i in 0..completed {}

            self.umem.completion_ring().release(completed);
        }

        0
    }
}

#[derive(Debug)]
pub enum SocketError {
    Mmap(MmapError),
    Umem(UmemError),
    Ring(RingError),
    InterfaceName(NulError),
    Initialize(std::io::Error),
    SocketIsNull,
    KickTx(std::io::Error),
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SocketError {}

impl From<MmapError> for SocketError {
    fn from(value: MmapError) -> Self {
        Self::Mmap(value)
    }
}

impl From<UmemError> for SocketError {
    fn from(value: UmemError) -> Self {
        Self::Umem(value)
    }
}

impl From<RingError> for SocketError {
    fn from(value: RingError) -> Self {
        Self::Ring(value)
    }
}
