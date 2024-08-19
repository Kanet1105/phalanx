use std::{
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

use libc::{
    mmap, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_HUGETLB, MAP_PRIVATE, PROT_READ, PROT_WRITE,
};

use crate::frame::Frame;

#[derive(Debug)]
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
            panic!("{:?}", MmapError::FreeMmap(std::io::Error::last_os_error()));
        }
    }
}

impl Mmap {
    pub fn new(
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
            return Err(MmapError::InitializeMmap(std::io::Error::last_os_error()));
        }

        Ok(Self {
            address: NonNull::new(address).ok_or(MmapError::MmapIsNull)?,
            frame_size,
            frame_headroom_size,
            descriptor_count,
        })
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut c_void {
        self.address.as_ptr()
    }

    #[inline(always)]
    pub fn offset(&self, count: isize) -> *mut c_void {
        unsafe { self.as_ptr().offset(count) }
    }

    #[inline(always)]
    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    #[inline(always)]
    pub fn frame_headroom_size(&self) -> u32 {
        self.frame_headroom_size
    }

    #[inline(always)]
    pub fn descriptor_count(&self) -> u32 {
        self.descriptor_count
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        ((self.frame_size + self.frame_headroom_size) * self.descriptor_count) as usize
    }

    pub fn populate(&self) -> Vec<Frame> {
        let mut frame_vec = Vec::<Frame>::with_capacity(self.descriptor_count as usize);
        let frame_size = self.frame_size + self.frame_headroom_size;

        for descriptor_index in 0..self.descriptor_count {
            let address = descriptor_index * frame_size;
            let offset = self.offset(address as isize) as *mut u8;

            let frame = Frame {
                address: offset as u64,
                length: 0,
                data: unsafe { std::slice::from_raw_parts_mut(offset, frame_size as usize) },
            };
            frame_vec.push(frame);
        }

        frame_vec
    }

    // pub fn populate(&self) -> Vec<Frame> {
    //     (0..self.descriptor_count)
    //         .map(|descriptor_index: u32| {
    //             let descriptor_offset = descriptor_index * self.frame_size;
    //             let mmap_address = self.offset(descriptor_offset as isize) as
    // *mut u8;

    //             Frame {
    //                 address: descriptor_offset as u64,
    //                 length: 0,
    //                 data: unsafe {
    //                     std::slice::from_raw_parts_mut(mmap_address,
    // self.frame_size as usize)                 },
    //             }
    //         })
    //         .collect()
    // }
}

#[derive(Debug)]
pub enum MmapError {
    InitializeMmap(std::io::Error),
    MmapIsNull,
    FreeMmap(std::io::Error),
}

impl std::fmt::Display for MmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MmapError {}
