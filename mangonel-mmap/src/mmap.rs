use std::{
    ffi::c_void,
    io,
    ptr::{self, NonNull},
};

use mangonel_core::{
    error::{Error, WrapError},
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
                .wrap("Failed to unmap the mmap region.")
                .unwrap()
        }
    }
}

impl Mmap {
    pub fn new(length: usize) -> Result<Self, Error> {
        let address = unsafe {
            let protection_mode = PROT_READ | PROT_WRITE;
            let flags = MAP_SHARED | MAP_ANONYMOUS;

            let address = libc::mmap(ptr::null_mut(), length, protection_mode, flags, -1, 0);

            if address == libc::MAP_FAILED {
                return Err(Error::boxed(
                    io::Error::last_os_error(),
                    "Failed to create a mmap region.",
                ));
            }

            address
        };

        Ok(Self {
            address: NonNull::new(address).unwrap(),
            length,
        })
    }

    #[cfg(features = "hugepages")]
    pub fn hugepages(length: usize) -> Result<Self, Error> {
        use mangonel_core::libc::MAP_HUGETLB;

        let address = unsafe {
            let protection_mode = PROT_READ | PROT_WRITE;
            let flags = MAP_SHARED | MAP_ANONYMOUS | MAP_HUGETLB;
            let address = libc::mmap(ptr::null_mut(), length, protection_mode, flags, -1, 0);

            if address == libc::MAP_FAILED {
                return Err(Error::boxed(
                    io::Error::last_os_error(),
                    "Failed to create a mmap region.",
                ));
            }
        };

        Ok(Self {
            address: NonNull::new(address).unwrap(),
            length,
        })
    }
}
