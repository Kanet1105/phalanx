use mangonel_libxdp_sys::xdp_desc;

use crate::socket::Socket;

pub struct Descriptor {
    address: u64,
    length: u32,
    socket: Socket,
}

impl From<(&xdp_desc, &Socket)> for Descriptor {
    fn from(value: (&xdp_desc, &Socket)) -> Self {
        unsafe {
            Self {
                address: (*value.0).addr,
                length: (*value.0).len,
                socket: value.1.clone(),
            }
        }
    }
}

impl Descriptor {
    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    #[inline(always)]
    pub fn get_data(&mut self) -> &mut [u8] {
        let headroom_size = self.socket.umem().umem_config().frame_headroom;
        let address = self.address - headroom_size as u64;
        let length = self.length + headroom_size;
        let offset = self.socket.umem().get_data(address) as *mut u8;

        unsafe { std::slice::from_raw_parts_mut(offset, length as usize) }
    }
}
