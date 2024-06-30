use std::{
    ffi::c_void,
    io,
    ptr::{self, NonNull},
};

use crate::{
    error::{Error, ErrorKind, WrapError},
    libc::{self, MAP_ANONYMOUS, MAP_HUGETLB, MAP_SHARED, PROT_READ, PROT_WRITE},
};

pub struct Mmap {
    address: NonNull<c_void>,
    length: usize,
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.address.as_ptr(), self.length)
                .wrap(ErrorKind::Munmap)
                .unwrap()
        }
    }
}

impl Mmap {
    pub fn new(length: usize) -> Result<Self, Error> {
        let protection_mode = PROT_READ | PROT_WRITE;
        let flags = MAP_SHARED | MAP_ANONYMOUS;
        let address = unsafe { libc::mmap(ptr::null_mut(), length, protection_mode, flags, -1, 0) };

        if address == libc::MAP_FAILED {
            return Err((ErrorKind::Mmap, io::Error::last_os_error()).into());
        }

        Ok(Self {
            address: NonNull::new(address).unwrap(),
            length,
        })
    }

    pub fn hugepages(length: usize) -> Result<Self, Error> {
        let protection_mode = PROT_READ | PROT_WRITE;
        let flags = MAP_SHARED | MAP_ANONYMOUS | MAP_HUGETLB;
        let address = unsafe { libc::mmap(ptr::null_mut(), length, protection_mode, flags, -1, 0) };

        if address == libc::MAP_FAILED {
            return Err((ErrorKind::Mmap, io::Error::last_os_error()).into());
        }

        Ok(Self {
            address: NonNull::new(address).unwrap(),
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
