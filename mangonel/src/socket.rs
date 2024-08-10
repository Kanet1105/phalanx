use std::{
    collections::VecDeque,
    ffi::{CString, NulError},
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use libc::{poll, pollfd, sendto, MSG_DONTWAIT, POLLIN, XDP_USE_NEED_WAKEUP};
use mangonel_libxdp_sys::{
    xsk_socket, xsk_socket__create, xsk_socket__delete, xsk_socket__fd, xsk_socket_config,
    xsk_socket_config__bindgen_ty_1, XDP_SHARED_UMEM, XSK_RING_CONS__DEFAULT_NUM_DESCS,
    XSK_RING_PROD__DEFAULT_NUM_DESCS, XSK_UMEM__DEFAULT_FRAME_HEADROOM,
    XSK_UMEM__DEFAULT_FRAME_SIZE,
};

use crate::{
    mmap::{Mmap, MmapError},
    packet::Packet,
    ring::{RingError, RxRing, TxRing},
    umem::{Umem, UmemError},
};

#[derive(Debug)]
pub struct SocketBuilder {
    pub frame_size: u32,
    pub frame_headroom: u32,
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
            frame_headroom: XSK_UMEM__DEFAULT_FRAME_HEADROOM,
            descriptor_count: XSK_RING_CONS__DEFAULT_NUM_DESCS * 2,
            completion_ring_size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            fill_ring_size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
            rx_ring_size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            tx_ring_size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
        }
    }
}

impl SocketBuilder {
    pub fn build(
        self,
        interface_name: impl AsRef<str>,
        queue_id: u32,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        let length = (self.frame_size + self.frame_headroom) * self.descriptor_count;
        let mmap = Mmap::initialize(length as usize)?;

        let umem = Umem::initialize(
            mmap,
            self.completion_ring_size,
            self.fill_ring_size,
            self.frame_size,
            self.frame_headroom,
        )
        .map_err(SocketError::Umem)?;
        umem.fill_ring().populate(self.frame_size)?;

        Socket::initialize(
            umem,
            self.rx_ring_size,
            self.tx_ring_size,
            interface_name,
            queue_id,
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
        umem: Umem,
        rx_ring_size: u32,
        tx_ring_size: u32,
        interface_name: impl AsRef<str>,
        queue_id: u32,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        let mut rx_ring = RxRing::uninitialized(rx_ring_size)?;
        let mut tx_ring = TxRing::uninitialized(tx_ring_size)?;
        let interface_name =
            CString::new(interface_name.as_ref()).map_err(SocketError::InterfaceName)?;
        let socket_config = xsk_socket_config {
            rx_size: rx_ring_size,
            tx_size: tx_ring_size,
            __bindgen_anon_1: xsk_socket_config__bindgen_ty_1 { libbpf_flags: 0 },
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
        let rx_socket = RxSocket::new(umem.clone(), socket.clone(), rx_ring.initialize()?);
        let tx_socket = TxSocket::new(umem.clone(), socket.clone(), tx_ring.initialize()?);

        Ok((rx_socket, tx_socket))
    }

    pub fn as_ptr(&self) -> *mut xsk_socket {
        self.inner.0.as_ptr()
    }

    pub fn socket_fd(&self) -> i32 {
        unsafe { xsk_socket__fd(self.as_ptr()) }
    }
}

pub struct RxSocket {
    umem: Umem,
    socket: Socket,
    rx_ring: RxRing,
}

impl RxSocket {
    pub fn new(umem: Umem, socket: Socket, rx_ring: RxRing) -> Self {
        Self {
            umem,
            socket,
            rx_ring,
        }
    }

    pub fn rx_burst(&mut self, buffer: &mut VecDeque<Packet>) -> u32 {
        0
    }
}

pub struct TxSocket {
    umem: Umem,
    socket: Socket,
    tx_ring: TxRing,
}

impl TxSocket {
    pub fn new(umem: Umem, socket: Socket, tx_ring: TxRing) -> Self {
        Self {
            umem,
            socket,
            tx_ring,
        }
    }

    pub fn tx_burst(&mut self, buffer: &mut VecDeque<Packet>) -> u32 {
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
