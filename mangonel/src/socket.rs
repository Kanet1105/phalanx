use std::{
    collections::VecDeque,
    ffi::{CString, NulError},
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use libc::{poll, pollfd, sendto, MSG_DONTWAIT, POLLIN};
use mangonel_libxdp_sys::{
    xsk_socket, xsk_socket__create, xsk_socket__delete, xsk_socket__fd, xsk_socket_config,
    xsk_socket_config__bindgen_ty_1, XDP_COPY, XDP_USE_NEED_WAKEUP, XDP_ZEROCOPY,
    XSK_RING_CONS__DEFAULT_NUM_DESCS, XSK_RING_PROD__DEFAULT_NUM_DESCS,
    XSK_UMEM__DEFAULT_FRAME_HEADROOM, XSK_UMEM__DEFAULT_FRAME_SIZE,
};

use crate::{
    frame::Descriptor,
    ring::{ConsumerRing, ProducerRing, RingError, RingType},
    umem::{Umem, UmemError},
    util::setrlimit,
};

#[derive(Debug)]
pub struct SocketBuilder {
    pub frame_size: u32,
    pub headroom_size: u32,
    pub descriptor_count: u32,
    pub completion_ring_size: u32,
    pub fill_ring_size: u32,
    pub rx_ring_size: u32,
    pub tx_ring_size: u32,
    pub use_hugetlb: bool,
    pub force_zero_copy: bool,
}

impl Default for SocketBuilder {
    fn default() -> Self {
        Self {
            frame_size: XSK_UMEM__DEFAULT_FRAME_SIZE,
            headroom_size: XSK_UMEM__DEFAULT_FRAME_HEADROOM,
            descriptor_count: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            completion_ring_size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            fill_ring_size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
            rx_ring_size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            tx_ring_size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
            use_hugetlb: false,
            force_zero_copy: false,
        }
    }
}

impl SocketBuilder {
    /// # Panics
    ///
    /// The function panics when [`setrlimit()`] panic conditions are met.
    pub fn build(
        self,
        interface_name: impl AsRef<str>,
        queue_id: u32,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        setrlimit();

        let umem = Umem::new(
            self.frame_size,
            self.headroom_size,
            self.descriptor_count,
            self.completion_ring_size,
            self.fill_ring_size,
            self.use_hugetlb,
        )?;

        let (rx_socket, tx_socket) = Socket::initialize(
            self.rx_ring_size,
            self.tx_ring_size,
            self.force_zero_copy,
            interface_name,
            queue_id,
            umem,
        )?;

        Ok((rx_socket, tx_socket))
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
    #[inline(always)]
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
        force_zero_copy: bool,
        interface_name: impl AsRef<str>,
        queue_id: u32,
        umem: Umem,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        let mut rx_ring = RingType::rx_ring_uninit(rx_ring_size)?;
        let mut tx_ring = RingType::tx_ring_uninit(tx_ring_size)?;

        let mut xdp_flags: u32 = XDP_USE_NEED_WAKEUP;
        match force_zero_copy {
            true => xdp_flags |= XDP_ZEROCOPY,
            false => xdp_flags |= XDP_COPY,
        }

        let interface_name =
            CString::new(interface_name.as_ref()).map_err(SocketError::InterfaceName)?;

        let socket_config = xsk_socket_config {
            rx_size: rx_ring_size,
            tx_size: tx_ring_size,
            __bindgen_anon_1: xsk_socket_config__bindgen_ty_1 { libxdp_flags: 0 },
            xdp_flags,
            bind_flags: 0,
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

        let rx_socket = RxSocket::new(socket.clone(), rx_ring.init()?, umem.clone());
        let tx_socket = TxSocket::new(socket.clone(), tx_ring.init()?, umem.clone());

        Ok((rx_socket, tx_socket))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut xsk_socket {
        self.inner.0.as_ptr()
    }

    #[inline(always)]
    pub fn socket_fd(&self) -> i32 {
        unsafe { xsk_socket__fd(self.inner.0.as_ptr()) }
    }

    #[inline(always)]
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
    rx_ring: ConsumerRing,
    umem: Umem,
}

impl RxSocket {
    pub fn new(socket: Socket, rx_ring: ConsumerRing, umem: Umem) -> Self {
        Self {
            socket,
            rx_ring,
            umem,
        }
    }

    #[inline(always)]
    pub fn socket(&self) -> &Socket {
        &self.socket
    }

    #[inline(always)]
    pub fn rx_ring(&self) -> &ConsumerRing {
        &self.rx_ring
    }

    #[inline(always)]
    pub fn umem(&self) -> &Umem {
        &self.umem
    }

    #[inline(always)]
    pub fn rx_burst(&mut self, buffer: &mut VecDeque<Descriptor>) -> u32 {
        self.umem.fill();

        let mut index: u32 = 0;
        let batch_size = buffer.capacity() as u32;

        let received = self.rx_ring.peek(batch_size, &mut index);
        if received > 0 {
            for _ in 0..received {
                let descriptor_ptr = self.rx_ring.rx_descriptor(index);
                let descriptor = Descriptor::from((descriptor_ptr, &self.umem));
                buffer.push_back(descriptor);
                index += 1;
            }

            self.rx_ring.release(received);
        }

        received
    }
}

pub struct TxSocket {
    socket: Socket,
    tx_ring: ProducerRing,
    umem: Umem,
}

impl TxSocket {
    pub fn new(socket: Socket, tx_ring: ProducerRing, umem: Umem) -> Self {
        Self {
            socket,
            tx_ring,
            umem,
        }
    }

    #[inline(always)]
    pub fn socket(&self) -> &Socket {
        &self.socket
    }

    #[inline(always)]
    pub fn tx_ring(&self) -> &ProducerRing {
        &self.tx_ring
    }

    #[inline(always)]
    pub fn umem(&self) -> &Umem {
        &self.umem
    }

    #[inline(always)]
    pub fn tx_burst(&mut self, buffer: &mut VecDeque<Descriptor>) -> u32 {
        let mut index: u32 = 0;
        let batch_size = buffer.len() as u32;

        let available = self.tx_ring.reserve(batch_size, &mut index);
        if available > 0 {
            for _ in 0..available {
                let descriptor = buffer.pop_front().unwrap();
                let descriptor_ptr = self.tx_ring.tx_descriptor(index);
                unsafe {
                    (*descriptor_ptr).addr = descriptor.address();
                    (*descriptor_ptr).len = descriptor.length();
                }
                index += 1;
            }

            self.tx_ring.submit(available);
        }

        if self.tx_ring.needs_wakeup() {
            unsafe {
                sendto(
                    self.socket.socket_fd(),
                    null_mut(),
                    0,
                    MSG_DONTWAIT,
                    null_mut(),
                    0,
                )
            };
        }
        self.umem.complete();

        available
    }
}

#[derive(Debug)]
pub enum SocketError {
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
