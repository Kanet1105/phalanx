use std::{
    ffi::c_void,
    ptr::{null_mut, NonNull},
    sync::Arc,
};

use crossbeam::queue::ArrayQueue;
use mangonel_libxdp_sys::{
    xsk_umem, xsk_umem__create, xsk_umem__delete, xsk_umem__get_data, xsk_umem_config,
    XDP_PACKET_HEADROOM,
};

use crate::{
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
    buffer_free: ArrayQueue<u64>,
    mmap: Mmap,
}

impl Drop for UmemInner {
    /// # Panics
    ///
    /// The program panics when it fails to clean up. This is not a problem
    /// while it is running and each [`RxSocket`] and [`TxSocket`] is referring
    /// to it. However, we want to see the error when it happens.
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

unsafe impl Send for Umem {}

unsafe impl Sync for Umem {}

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
        headroom_size: u32,
        fill_ring_size: u32,
        completion_ring_size: u32,
        use_hugetlb: bool,
    ) -> Result<Self, UmemError> {
        let mmap = Mmap::new(frame_size, headroom_size, fill_ring_size, use_hugetlb)?;

        let mut umem_ptr = null_mut::<xsk_umem>();
        let mut fill_ring = ProducerRingUninit::new(fill_ring_size)?;
        let mut completion_ring = ConsumerRingUninit::new(completion_ring_size)?;
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

        // Pre-fill the buffer with addresses.
        let buffer_free = ArrayQueue::new(fill_ring_size as usize);
        (0..fill_ring_size).for_each(|descriptor_index: u32| {
            let offset = descriptor_index * (frame_size + headroom_size + XDP_PACKET_HEADROOM);
            buffer_free.push(offset as u64).unwrap();
        });

        let inner = UmemInner {
            umem: NonNull::new(umem_ptr).ok_or(UmemError::UmemIsNull)?,
            fill_ring: fill_ring.init()?,
            completion_ring: completion_ring.init()?,
            buffer_free,
            mmap,
        };
        let umem = Self {
            inner: Arc::new(inner),
        };

        Ok(umem)
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut xsk_umem {
        self.inner.umem.as_ptr()
    }

    #[inline(always)]
    pub fn fill_ring(&self) -> &ProducerRing {
        &self.inner.fill_ring
    }

    #[inline(always)]
    pub fn completion_ring(&self) -> &ConsumerRing {
        &self.inner.completion_ring
    }

    #[inline(always)]
    pub fn buffer_free(&self) -> &ArrayQueue<u64> {
        &self.inner.buffer_free
    }

    #[inline(always)]
    pub fn mmap(&self) -> &Mmap {
        &self.inner.mmap
    }

    #[inline(always)]
    pub fn fill(&self) -> u32 {
        let mut index: u32 = 0;
        let size = self.inner.buffer_free.len() as u32;

        let available = self.inner.fill_ring.reserve(size, &mut index);
        if available > 0 {
            for _ in 0..available {
                let address = self.inner.fill_ring.fill_address(index);
                unsafe { *address = self.inner.buffer_free.pop().unwrap() }
                index += 1;
            }

            self.inner.fill_ring.submit(available);
        }

        available
    }

    #[inline(always)]
    pub fn complete(&self, size: u32) -> u32 {
        let mut index: u32 = 0;

        let available = self.inner.completion_ring.peek(size, &mut index);
        if available > 0 {
            for _ in 0..available {
                let address = self.inner.completion_ring.complete_address(index);
                unsafe { self.inner.buffer_free.push(*address).unwrap() };
                index += 1;
            }

            self.inner.completion_ring.release(available);
        }

        available
    }

    #[inline(always)]
    pub fn get_data(&self, address: u64) -> *mut c_void {
        unsafe { xsk_umem__get_data(self.inner.mmap.as_ptr(), address) }
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
