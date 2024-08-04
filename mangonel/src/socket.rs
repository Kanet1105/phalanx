use std::ptr::NonNull;

use mangonel_libxdp_sys::{
    xsk_ring_cons, xsk_ring_cons__peek, xsk_ring_cons__release, xsk_ring_cons__rx_desc,
    xsk_ring_prod, xsk_ring_prod__reserve,
};

pub struct Receiver {
    ptr: NonNull<xsk_ring_cons>,
}

unsafe impl Send for Receiver {}

impl Receiver {
    pub fn receive(&mut self, buffer: &mut Vec<u8>, batch_size: u32) -> u32 {
        let mut index: u32 = 0;
        let received = unsafe { xsk_ring_cons__peek(self.as_ptr(), batch_size, &mut index) };
        if received == 0 {
            return received;
        }

        for _ in 0..received {
            // let mut packet = [0; ];
            unsafe {
                let descriptor = xsk_ring_cons__rx_desc(self.as_ptr(), index);
                let address = (*descriptor).addr;
                let length = (*descriptor).len;
            }

            // buffer.push();
            index += 1;
        }

        unsafe { xsk_ring_cons__release(self.as_ptr(), received) }

        received
    }

    fn as_ptr(&self) -> *mut xsk_ring_cons {
        self.ptr.as_ptr()
    }
}

#[derive(Debug)]
pub enum ReceiverError {}

pub struct Sender {
    ptr: NonNull<xsk_ring_prod>,
}

impl Sender {
    pub fn send(&mut self, buffer: &mut Vec<u8>) {
        let mut index: u32 = 0;
        let batch_size = buffer.len() as u32;

        let available = unsafe { xsk_ring_prod__reserve(self.as_ptr(), batch_size, &mut index) };
        for _ in 0..available {
            // let packet
        }
    }

    fn as_ptr(&self) -> *mut xsk_ring_prod {
        self.ptr.as_ptr()
    }
}
