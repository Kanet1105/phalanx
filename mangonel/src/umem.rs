use std::{
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use mangonel_libxdp_sys::{xsk_umem, xsk_umem__create, xsk_umem__delete, xsk_umem_config};

use crate::{
    mmap::{Mmap, MmapError},
    ring::{CompletionRing, FillRing, RingError},
};

pub struct Umem(Arc<UmemInner>);

impl std::ops::Deref for Umem {
    type Target = UmemInner;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Umem {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Umem {
    pub fn new(
        frame_size: u32,
        headroom_size: u32,
        descriptor_count: u32,
        completion_ring_size: u32,
        fill_ring_size: u32,
        use_hugetlb: bool,
    ) -> Result<Self, UmemError> {
        let mmap = Mmap::new(frame_size, headroom_size, descriptor_count, use_hugetlb)?;

        let mut umem_ptr = null_mut::<xsk_umem>();
        let mut completion_ring = CompletionRing::uninitialized(completion_ring_size)?;
        let mut fill_ring = FillRing::uninitialized(fill_ring_size)?;
        let umem_config = xsk_umem_config {
            fill_size: fill_ring_size,
            comp_size: completion_ring_size,
            frame_size,
            frame_headroom: headroom_size,
            flags: 0,
        };

        let value = unsafe {
            xsk_umem__create(
                &mut umem_ptr,
                mmap.as_ptr(),
                mmap.length().try_into().unwrap(),
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

        let inner = UmemInner {
            umem: NonNull::new(umem_ptr).ok_or(UmemError::UmemIsNull)?,
            completion_ring: completion_ring.initialize()?,
            fill_ring: fill_ring.initialize()?,
            mmap,
        };

        Ok(Self(Arc::new(inner)))
    }
}

pub struct UmemInner {
    umem: NonNull<xsk_umem>,
    completion_ring: CompletionRing,
    fill_ring: FillRing,
    mmap: Mmap,
}

impl Drop for UmemInner {
    /// # Panics
    ///
    /// [`Umem`] will panic when it fails to clean up. This is not a problem
    /// while the program is running and each [`RxSocket`] and [`TxSocket`] is
    /// referring to it. However, we want to see the error when it happens.
    fn drop(&mut self) {
        let value = unsafe { xsk_umem__delete(self.umem.as_ptr()) };
        if value.is_negative() {
            panic!(
                "{:?}",
                UmemError::Free(std::io::Error::from_raw_os_error(-value))
            );
        }
    }
}

impl UmemInner {
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut xsk_umem {
        self.umem.as_ptr()
    }

    #[inline(always)]
    pub fn completion_ring(&self) -> &CompletionRing {
        &self.completion_ring
    }

    #[inline(always)]
    pub fn fill_ring(&self) -> &FillRing {
        &self.fill_ring
    }

    #[inline(always)]
    pub fn mmap(&self) -> &Mmap {
        &self.mmap
    }
}

#[derive(Debug)]
pub enum UmemError {
    Mmap(MmapError),
    Ring(RingError),
    Initialize(std::io::Error),
    UmemIsNull,
    Free(std::io::Error),
}

impl std::fmt::Display for UmemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for UmemError {}

impl From<MmapError> for UmemError {
    fn from(value: MmapError) -> Self {
        Self::Mmap(value)
    }
}

impl From<RingError> for UmemError {
    fn from(value: RingError) -> Self {
        Self::Ring(value)
    }
}
