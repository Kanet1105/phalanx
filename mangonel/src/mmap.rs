use std::{
    ffi::c_void,
    ptr::{self, NonNull},
    sync::Arc,
};

use libc::{self, MAP_ANONYMOUS, MAP_HUGETLB, MAP_SHARED, PROT_READ, PROT_WRITE};

pub struct Mmap {
    inner: Arc<MmapInner>,
}

struct MmapInner {
    address: NonNull<c_void>,
    length: usize,
}

impl Clone for Mmap {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe {
            let value = libc::munmap(self.inner.address.as_ptr(), self.inner.length);
            if value.is_negative() {
                panic!("{:?}", MmapError::Unmap);
            }
        }
    }
}

impl Mmap {
    pub fn new(length: usize) -> Result<Self, MmapError> {
        let protection_mode = PROT_READ | PROT_WRITE;
        let flags = MAP_SHARED | MAP_ANONYMOUS | MAP_HUGETLB;

        let address = unsafe { libc::mmap(ptr::null_mut(), length, protection_mode, flags, -1, 0) };
        if address == libc::MAP_FAILED {
            return Err(MmapError::Map(std::io::Error::last_os_error()));
        }

        let inner = MmapInner {
            address: NonNull::new(address).ok_or(MmapError::MmapAddressIsNull)?,
            length,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.inner.address.as_ptr()
    }

    pub fn length(&self) -> usize {
        self.inner.length
    }
}

#[derive(Debug)]
pub enum MmapError {
    Map(std::io::Error),
    Unmap,
    MmapAddressIsNull,
}

impl std::fmt::Display for MmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MmapError {}
