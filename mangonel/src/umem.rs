use std::{
    ffi::c_void,
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use mangonel_libxdp_sys::{
    xsk_umem, xsk_umem__create, xsk_umem__delete, xsk_umem__get_data, xsk_umem_config,
};

use crate::{
    buffer::Buffer,
    mmap::{Mmap, MmapError},
    ring::{ConsumerRing, ConsumerRingUninit, ProducerRing, ProducerRingUninit, RingError},
};

pub struct Umem {
    inner: Arc<UmemInner>,
}

struct UmemInner {
    umem: NonNull<xsk_umem>,
    completion_ring: ConsumerRing,
    fill_ring: ProducerRing,
    mmap: Mmap,
}

unsafe impl Send for UmemInner {}

unsafe impl Sync for UmemInner {}

impl Drop for UmemInner {
    /// # Panics
    ///
    /// The program panics when it fails to clean up. This is not a problem
    /// while it is running and either [`RxSocket`] and [`TxSocket`] is
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

impl Clone for Umem {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Umem {
    pub fn new(
        frame_size: u32,
        frame_headroom_size: u32,
        ring_size: u32,
        use_hugetlb: bool,
    ) -> Result<Self, UmemError> {
        let length = frame_size * ring_size;
        let mmap = Mmap::new(length as usize, use_hugetlb)?;

        let mut umem_ptr = null_mut::<xsk_umem>();
        let mut fill_ring = ProducerRingUninit::new(ring_size)?;
        let mut completion_ring = ConsumerRingUninit::new(ring_size)?;
        let umem_config = xsk_umem_config {
            fill_size: ring_size,
            comp_size: ring_size,
            frame_size,
            frame_headroom: frame_headroom_size,
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
            fill_ring: fill_ring.init()?,
            completion_ring: completion_ring.init()?,
            mmap,
        };
        let umem = Self {
            inner: Arc::new(inner),
        };

        Ok(umem)
    }

    #[inline(always)]
    pub(crate) fn as_ptr(&self) -> *mut xsk_umem {
        self.inner.umem.as_ptr()
    }

    #[inline(always)]
    pub(crate) fn get_data(&self, address: u64) -> *mut c_void {
        unsafe { xsk_umem__get_data(self.inner.mmap.as_ptr(), address) }
    }

    #[inline(always)]
    pub fn needs_wakeup(&self) -> bool {
        self.inner.fill_ring.needs_wakeup()
    }

    #[inline(always)]
    pub fn fill<T: Buffer<u64>>(&self, buffer: &T) -> u32 {
        let mut index: u32 = 0;
        let size = std::cmp::min(buffer.count(), self.inner.fill_ring.size);

        let available = self.inner.fill_ring.reserve(size, &mut index);
        if available > 0 {
            for _ in 0..available {
                let address = self.inner.fill_ring.fill_address(index);
                unsafe {
                    *address = buffer.pop().unwrap();
                }
                index += 1;
            }

            self.inner.fill_ring.submit(available);
        }

        available
    }

    #[inline(always)]
    pub fn complete<T: Buffer<u64>>(&self, buffer: &T) -> u32 {
        let mut index: u32 = 0;
        let size = std::cmp::min(buffer.free(), self.inner.completion_ring.size);

        let available = self.inner.completion_ring.peek(size, &mut index);
        if available > 0 {
            for _ in 0..available {
                let address = self.inner.completion_ring.complete_address(index);
                unsafe {
                    buffer.push(*address);
                }
                index += 1;
            }

            self.inner.completion_ring.release(available);
        }

        available
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
