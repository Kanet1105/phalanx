use std::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use libc::{self, MAP_ANONYMOUS, MAP_HUGETLB, MAP_PRIVATE, PROT_READ, PROT_WRITE};

pub struct Mmap {
    address: NonNull<c_void>,
    length: usize,
}

impl Drop for Mmap {
    fn drop(&mut self) {
        let value = unsafe { libc::munmap(self.as_ptr(), self.length()) };
        if value.is_negative() {
            panic!("{:?}", MmapError::Unmap(std::io::Error::last_os_error()));
        }
    }
}

impl Mmap {
    pub fn new(packet_size: usize, buffer_length: usize) -> Result<Self, MmapError> {
        let protection_mode = PROT_READ | PROT_WRITE;
        let flags = MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB;
        let length = packet_size * buffer_length;

        let address = unsafe { libc::mmap(ptr::null_mut(), length, protection_mode, flags, -1, 0) };
        if address == libc::MAP_FAILED {
            return Err(MmapError::Map(std::io::Error::last_os_error()));
        }

        Ok(Self {
            address: NonNull::new(address).ok_or(MmapError::MmapAddressIsNull)?,
            length,
        })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.address.as_ptr()
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

#[derive(Debug)]
pub enum MmapError {
    Map(std::io::Error),
    Unmap(std::io::Error),
    MmapAddressIsNull,
}

impl std::fmt::Display for MmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MmapError {}
