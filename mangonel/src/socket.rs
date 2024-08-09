use std::{
    collections::VecDeque,
    ffi::{CString, NulError},
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use libc::{poll, pollfd, sendto, MSG_DONTWAIT, POLLIN, XDP_USE_NEED_WAKEUP};
use mangonel_libxdp_sys::{
    xsk_ring_cons, xsk_ring_cons__peek, xsk_ring_cons__release, xsk_ring_cons__rx_desc,
    xsk_ring_prod, xsk_ring_prod__needs_wakeup, xsk_ring_prod__reserve, xsk_ring_prod__submit,
    xsk_ring_prod__tx_desc, xsk_socket, xsk_socket__create, xsk_socket__delete, xsk_socket__fd,
    xsk_socket_config, xsk_socket_config__bindgen_ty_1, XDP_SHARED_UMEM,
    XSK_RING_CONS__DEFAULT_NUM_DESCS, XSK_RING_PROD__DEFAULT_NUM_DESCS,
    XSK_UMEM__DEFAULT_FRAME_HEADROOM, XSK_UMEM__DEFAULT_FRAME_SHIFT, XSK_UMEM__DEFAULT_FRAME_SIZE,
};

use crate::{
    mmap::{Mmap, MmapError},
    packet::Packet,
    ring::{CompletionRing, FillRing, RingError, RxRing, TxRing},
    umem::{Umem, UmemError},
};

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
            descriptor_count: 8192,
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

        let completion_ring = CompletionRing::uninitialized(self.completion_ring_size)?;
        let fill_ring = FillRing::uninitialized(self.fill_ring_size)?;

        let umem = Umem::initialize(
            &mmap,
            &completion_ring,
            &fill_ring,
            self.frame_size,
            self.frame_headroom,
        )
        .map_err(SocketError::Umem)?;

        let rx_ring = RxRing::uninitialized(self.rx_ring_size)?;
        let tx_ring = TxRing::uninitialized(self.tx_ring_size)?;

        Socket::initialize(
            mmap,
            completion_ring,
            fill_ring,
            umem,
            rx_ring,
            tx_ring,
            interface_name,
            queue_id,
        )
    }
}

pub struct RxSocket;

pub struct TxSocket;

pub struct Socket {
    inner: Arc<SocketInner>,
}

struct SocketInner {
    mmap: Mmap,
    completion_ring: CompletionRing,
    fill_ring: FillRing,
    umem: Umem,
    rx_ring: RxRing,
    tx_ring: TxRing,
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
    pub fn initialize(
        mmap: Mmap,
        completion_ring: CompletionRing,
        fill_ring: FillRing,
        umem: Umem,
        rx_ring: RxRing,
        tx_ring: TxRing,
        interface_name: impl AsRef<str>,
        queue_id: u32,
    ) -> Result<(RxSocket, TxSocket), SocketError> {
        let socket = NonNull::<xsk_socket>::dangling();
        let interface_name =
            CString::new(interface_name.as_ref()).map_err(SocketError::InterfaceName)?;
        let socket_config = xsk_socket_config {
            rx_size: rx_ring.size(),
            tx_size: tx_ring.size(),
            __bindgen_anon_1: xsk_socket_config__bindgen_ty_1 { libxdp_flags: 0 },
            xdp_flags: XDP_SHARED_UMEM,
            bind_flags: XDP_USE_NEED_WAKEUP,
        };

        let value = unsafe {
            xsk_socket__create(
                &mut socket.as_ptr(),
                interface_name.as_ptr(),
                queue_id,
                umem.as_ptr(),
                rx_ring.as_ptr(),
                tx_ring.as_ptr(),
                &socket_config,
            )
        };

        if value.is_negative() {
            return Err(SocketError::Initialize(std::io::Error::from_raw_os_error(
                -value,
            )));
        }

        let inner = SocketInner {
            mmap,
            completion_ring,
            fill_ring,
            umem,
            rx_ring,
            tx_ring,
            socket,
        };

        let rx_socket = RxSocket;
        let tx_socket = TxSocket;

        Ok((rx_socket, tx_socket))
    }
}

// impl Socket {
//     pub fn new(
//         packet_size: usize,
//         buffer_length: usize,
//         queue_id: u32,
//         rx_ring_size: u32,
//         tx_ring_size: u32,
//         interface_name: impl AsRef<str>,
//     ) -> Result<(Receiver, Sender), SocketError> { let mmap =
//       Mmap::new(packet_size * buffer_length);

//         let completion_ring = CompletionRing::new(rx_ring_size);
//         let

//         let umem = Mmap::new();
//         let umem = Umem::new(packet_size, buffer_length, rx_ring_size,
// tx_ring_size).unwrap();

//         let socket_config = xsk_socket_config {
//             rx_size: rx_ring_size,
//             tx_size: tx_ring_size,
//             __bindgen_anon_1: xsk_socket_config__bindgen_ty_1 { libbpf_flags:
// 0 },             xdp_flags: XDP_SHARED_UMEM,
//             bind_flags: XDP_USE_NEED_WAKEUP,
//         };

//         let mut socket: *mut xsk_socket = null_mut();

//         let interface_name =
//
// CString::new(interface_name.as_ref()).map_err(SocketError::InterfaceName)?;

//         let mut rx_ring = MaybeUninit::<xsk_ring_cons>::zeroed();
//         let mut tx_ring = MaybeUninit::<xsk_ring_prod>::zeroed();

//         let value = unsafe {
//             xsk_socket__create(
//                 &mut socket,
//                 interface_name.as_ptr(),
//                 queue_id,
//                 umem.as_ptr(),
//                 rx_ring.as_mut_ptr(),
//                 tx_ring.as_mut_ptr(),
//                 &socket_config,
//             )
//         };

//         if value.is_negative() {
//             return
// Err(SocketError::Initialize(std::io::Error::from_raw_os_error(
// -value,             )));
//         }

//         let rx_ring = unsafe { rx_ring.assume_init() };
//         let tx_ring = unsafe { tx_ring.assume_init() };

//         let socket = Socket {
//             inner: Arc::new(SocketInner {
//                 umem,
//                 socket:
// NonNull::new(socket).ok_or(SocketError::SocketIsNull)?,             }),
//         };

//         let receiver = Receiver::new(rx_ring, socket.clone());
//         let sender = Sender::new(tx_ring, socket.clone());

//         Ok((receiver, sender))
//     }

//     pub fn as_ptr(&self) -> *mut xsk_socket {
//         self.inner.socket.as_ptr()
//     }

//     pub fn umem(&self) -> &Umem {
//         &self.inner.umem
//     }
// }

// pub struct Receiver {
//     rx_ring: xsk_ring_cons,
//     socket: Socket,
// }

// unsafe impl Send for Receiver {}

// impl Receiver {
//     pub fn new(rx_ring: xsk_ring_cons, socket: Socket) -> Self {
//         Self { socket, rx_ring }
//     }

//     pub fn rx_burst(&mut self, buffer: &mut VecDeque<Packet>) -> u32 {
//         let mut index: u32 = 0;
//         let burst_size = buffer.capacity() as u32;

//         let received = unsafe { xsk_ring_cons__peek(&mut self.rx_ring,
// burst_size, &mut index) };         if received == 0 {
//             self.wakeup();

//             return received;
//         }

//         for _ in 0..received {
//             unsafe {
//                 let descriptor = xsk_ring_cons__rx_desc(&mut self.rx_ring,
// index);                 let address = (*descriptor).addr;
//                 let length = (*descriptor).len;

//                 let packet = Packet {
//                     address,
//                     length,
//                     // data: std::slice::from_raw_parts_mut(
//                     //     self.socket.umem().offset(address as isize) as
// *mut u8,                     //     self.socket.umem().packet_size(),
//                     // ),
//                 };

//                 buffer.push_back(packet);
//                 index += 1;
//             }
//         }

//         unsafe { xsk_ring_cons__release(&mut self.rx_ring, received) }

//         received
//     }

//     fn wakeup(&self) {
//         unsafe {
//             let needs_wakeup =
// xsk_ring_prod__needs_wakeup(self.socket.umem().fill_ring_as_ptr());
//             if needs_wakeup > 0 {
//                 let mut poll_fd = pollfd {
//                     fd: xsk_socket__fd(self.socket.as_ptr()),
//                     events: POLLIN,
//                     revents: 0,
//                 };

//                 poll(&mut poll_fd, 1, 0);
//             }
//         }
//     }
// }

// pub struct Sender {
//     tx_ring: xsk_ring_prod,
//     socket: Socket,
// }

// unsafe impl Send for Sender {}

// impl Sender {
//     pub fn new(tx_ring: xsk_ring_prod, socket: Socket) -> Self {
//         Self { tx_ring, socket }
//     }

//     pub fn tx_burst(&mut self, buffer: &mut VecDeque<Packet>) -> Result<u32,
// SocketError> {         let mut index: u32 = 0;
//         let burst_size = buffer.len() as u32;

//         let value = unsafe { xsk_ring_prod__reserve(&mut self.tx_ring,
// burst_size, &mut index) };         if value != burst_size {
//             for packet_index in 0..burst_size {
//                 mem_
//             }
//         }
//     }

//     // fn complete_tx(&self) -> Result<(), SocketError> {
//     //     let mut index: u32 = 0;
//     //     self.kick_tx()?;

//     //     let completed = unsafe {
//     //         xsk_ring_cons__peek(
//     //             self.socket.umem().completion_ring_as_ptr(),
//     //             4096,
//     //             &mut index,
//     //         )
//     //     };

//     //     if completed > 0 {}

//     //     Ok(0)
//     // }

//     fn kick_tx(&self) -> Result<(), SocketError> {
//         let value = unsafe {
//             sendto(
//                 xsk_socket__fd(self.socket.as_ptr()),
//                 null_mut(),
//                 0,
//                 MSG_DONTWAIT,
//                 null_mut(),
//                 0,
//             )
//         };

//         if value.is_negative() {
//             return Err(SocketError::KickTx(std::io::Error::from_raw_os_error(
//                 -value as i32,
//             )));
//         }

//         Ok(())
//     }

//     // pub fn tx_burst(&mut self, buffer: &mut VecDeque<Packet>) -> u32 {
//     //     let mut index: u32 = 0;
//     //     let batch_size = buffer.capacity() as u32;

//     //     if batch_size == 0 {
//     //         return 0;
//     //     }

//     //     let completed =
//     //         unsafe { xsk_ring_prod__reserve(&mut self.tx_ring, batch_size,
// &mut     // index) };     for _ in 0..available {
//     //         let packet = buffer.pop_front().unwrap();

//     //         unsafe {
//     //             let descriptor = xsk_ring_prod__tx_desc(&mut self.tx_ring,
//     // available);             (*descriptor).addr = packet.address;
//     //             (*descriptor).len = packet.length;
//     //             index += 1;
//     //         }
//     //     }

//     //     if available > 0 {
//     //         unsafe { xsk_ring_prod__submit(&mut self.tx_ring, available)
// };     //     }

//     //     return 0;
//     // }
// }

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
