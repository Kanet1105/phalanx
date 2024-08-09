use std::ptr::NonNull;

use mangonel_libxdp_sys::{xsk_umem, xsk_umem__create, xsk_umem_config};

use crate::{
    mmap::Mmap,
    ring::{CompletionRing, FillRing},
};

pub struct Umem(NonNull<xsk_umem>);

impl Umem {
    pub fn initialize(
        mmap: &Mmap,
        completion_ring: &CompletionRing,
        fill_ring: &FillRing,
        frame_size: u32,
        frame_headroom: u32,
    ) -> Result<Self, UmemError> {
        let umem_config = xsk_umem_config {
            fill_size: fill_ring.size(),
            comp_size: completion_ring.size(),
            frame_size,
            frame_headroom,
            flags: 0,
        };
        let umem: NonNull<xsk_umem> = NonNull::dangling();

        let value = unsafe {
            xsk_umem__create(
                &mut umem.as_ptr(),
                mmap.as_ptr(),
                mmap.length() as u64,
                fill_ring.as_ptr(),
                completion_ring.as_ptr(),
                &umem_config,
            )
        };

        if value.is_negative() {
            let error = std::io::Error::from_raw_os_error(-value);

            return Err(UmemError::Initialize(error));
        }

        Ok(Self(umem))
    }

    pub fn as_ptr(&self) -> *mut xsk_umem {
        self.0.as_ptr()
    }
}

pub enum UmemError {
    Initialize(std::io::Error),
}

impl std::fmt::Debug for UmemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialize(error) => write!(f, "Failed to initialize Umem: {:?}", error),
        }
    }
}

impl std::fmt::Display for UmemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for UmemError {}
