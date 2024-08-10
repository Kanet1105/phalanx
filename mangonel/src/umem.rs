use std::{
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use mangonel_libxdp_sys::{xsk_umem, xsk_umem__create, xsk_umem__delete, xsk_umem_config};

use crate::{
    mmap::Mmap,
    ring::{CompletionRing, FillRing, RingError},
};

pub struct Umem {
    inner: Arc<UmemInner>,
}

struct UmemInner {
    mmap: Mmap,
    completion_ring: CompletionRing,
    fill_ring: FillRing,
    umem: NonNull<xsk_umem>,
}

impl Drop for UmemInner {
    fn drop(&mut self) {
        let value = unsafe { xsk_umem__delete(self.umem.as_ptr()) };
        if value.is_negative() {
            panic!(
                "{:?}",
                UmemError::CleanUp(std::io::Error::from_raw_os_error(-value))
            );
        }
    }
}

unsafe impl Send for Umem {}

unsafe impl Sync for Umem {}

impl Clone for Umem {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Umem {
    pub fn initialize(
        mmap: Mmap,
        completion_ring_size: u32,
        fill_ring_size: u32,
        frame_size: u32,
        frame_headroom: u32,
    ) -> Result<Self, UmemError> {
        let mut completion_ring = CompletionRing::uninitialized(fill_ring_size)?;
        let mut fill_ring = FillRing::uninitialized(fill_ring_size)?;
        let umem_config = xsk_umem_config {
            fill_size: fill_ring_size,
            comp_size: completion_ring_size,
            frame_size,
            frame_headroom,
            flags: 0,
        };
        let mut umem_ptr = null_mut();

        let value = unsafe {
            xsk_umem__create(
                &mut umem_ptr,
                mmap.as_ptr(),
                mmap.length() as u64,
                fill_ring.as_mut_ptr(),
                completion_ring.as_mut_ptr(),
                &umem_config,
            )
        };
        if value.is_negative() {
            return Err(UmemError::Initialize(std::io::Error::from_raw_os_error(
                -value,
            )));
        }

        let umem_inner = UmemInner {
            mmap,
            completion_ring: completion_ring.initialize()?,
            fill_ring: fill_ring.initialize()?,
            umem: NonNull::new(umem_ptr).unwrap(),
        };

        Ok(Self {
            inner: Arc::new(umem_inner),
        })
    }

    pub fn mmap(&self) -> &Mmap {
        &self.inner.mmap
    }

    pub fn completion_ring(&self) -> &CompletionRing {
        &self.inner.completion_ring
    }

    pub fn fill_ring(&self) -> &FillRing {
        &self.inner.fill_ring
    }

    pub fn as_ptr(&self) -> *mut xsk_umem {
        self.inner.umem.as_ptr()
    }
}

#[derive(Debug)]
pub enum UmemError {
    Ring(RingError),
    Initialize(std::io::Error),
    CleanUp(std::io::Error),
}

impl std::fmt::Display for UmemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for UmemError {}

impl From<RingError> for UmemError {
    fn from(value: RingError) -> Self {
        Self::Ring(value)
    }
}
