use mangonel_libxdp_sys::xdp_desc;
use pnet::packet::{ethernet::MutableEthernetPacket, ipv4::MutableIpv4Packet};

use crate::umem::Umem;

pub struct Descriptor {
    address: u64,
    length: u32,
    umem: Umem,
}

impl From<(*const xdp_desc, &Umem)> for Descriptor {
    #[inline(always)]
    fn from(value: (*const xdp_desc, &Umem)) -> Self {
        unsafe {
            Self {
                address: (*value.0).addr,
                length: (*value.0).len,
                umem: value.1.clone(),
            }
        }
    }
}

impl Descriptor {
    #[inline(always)]
    pub fn address(&self) -> u64 {
        self.address
    }

    #[inline(always)]
    pub fn length(&self) -> u32 {
        self.length
    }

    #[inline(always)]
    pub fn get_data(&mut self) {
        let offset = self.umem.get_data(self.address) as *mut u8;
        let data = unsafe { std::slice::from_raw_parts_mut(offset, self.length as usize) };
        // let ethernet_frame = MutableEthernetPacket::new(data).unwrap();
        let ipv4_packet = MutableIpv4Packet::new(&mut data[14..]).unwrap();
        println!("{:?}", ipv4_packet);
    }
}
