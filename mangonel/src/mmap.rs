use std::{
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

use libc::{
    mmap, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_HUGETLB, MAP_PRIVATE, PROT_READ, PROT_WRITE,
};

use crate::packet::Frame;

pub struct Mmap {
    address: NonNull<c_void>,
    frame_size: u32,
    frame_headroom_size: u32,
    descriptor_count: u32,
}

impl Drop for Mmap {
    fn drop(&mut self) {
        let value = unsafe { munmap(self.as_ptr(), self.length()) };
        if value.is_negative() {
            panic!("{:?}", MmapError::Unmap(std::io::Error::last_os_error()));
        }
    }
}

impl Mmap {
    pub fn initialize(
        frame_size: u32,
        frame_headroom_size: u32,
        descriptor_count: u32,
        hugetlb: bool,
    ) -> Result<Self, MmapError> {
        let protection_mode = PROT_READ | PROT_WRITE;
        let mut flags = MAP_PRIVATE | MAP_ANONYMOUS;
        if hugetlb {
            flags |= MAP_HUGETLB;
        }
        let length = (frame_size * descriptor_count) as usize;

        let address = unsafe { mmap(null_mut(), length, protection_mode, flags, -1, 0) };
        if address == MAP_FAILED {
            return Err(MmapError::Map(std::io::Error::last_os_error()));
        }

        Ok(Self {
            address: NonNull::new(address).ok_or(MmapError::MmapAddressIsNull)?,
            frame_size,
            frame_headroom_size,
            descriptor_count,
        })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.address.as_ptr()
    }

    pub fn offset(&self, count: isize) -> *mut c_void {
        unsafe { self.as_ptr().offset(count) }
    }

    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    pub fn frame_headroom_size(&self) -> u32 {
        self.frame_headroom_size
    }

    pub fn descriptor_count(&self) -> u32 {
        self.descriptor_count
    }

    pub fn length(&self) -> usize {
        ((self.frame_size + self.frame_headroom_size) * self.descriptor_count) as usize
    }

    pub fn initialize_descriptor_buffer(&self) -> Vec<Frame> {
        let mut buffer = Vec::<Frame>::with_capacity(self.descriptor_count as usize);

        for i in 0..self.descriptor_count {
            unsafe {
                let address = (i * (self.frame_size + self.frame_headroom_size)) as u64;
                let mmap_ptr = self.as_ptr().offset(address as isize) as *mut u8;
                let descriptor = Frame {
                    address,
                    length: 0,
                    data: std::slice::from_raw_parts_mut(mmap_ptr, self.frame_size as usize),
                };

                buffer.push(descriptor);
            }
        }

        buffer
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
