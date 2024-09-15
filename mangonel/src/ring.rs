use std::{mem::MaybeUninit, ptr::NonNull, sync::Arc};

use crossbeam::queue::ArrayQueue;
use mangonel_libxdp_sys::{
    xdp_desc, xsk_ring_cons, xsk_ring_cons__cancel, xsk_ring_cons__comp_addr, xsk_ring_cons__peek,
    xsk_ring_cons__release, xsk_ring_cons__rx_desc, xsk_ring_prod, xsk_ring_prod__fill_addr,
    xsk_ring_prod__needs_wakeup, xsk_ring_prod__reserve, xsk_ring_prod__submit,
    xsk_ring_prod__tx_desc, XSK_RING_CONS__DEFAULT_NUM_DESCS, XSK_RING_PROD__DEFAULT_NUM_DESCS,
};

use crate::{descriptor::Descriptor, util::is_power_of_two};

pub struct ConsumerRingUninit {
    size: u32,
    ring: Box<MaybeUninit<xsk_ring_cons>>,
}

impl Default for ConsumerRingUninit {
    fn default() -> Self {
        Self {
            size: XSK_RING_CONS__DEFAULT_NUM_DESCS,
            ring: Box::new(MaybeUninit::<xsk_ring_cons>::uninit()),
        }
    }
}

impl ConsumerRingUninit {
    pub fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(size));
        }

        Ok(Self {
            size,
            ring: Box::new(MaybeUninit::<xsk_ring_cons>::uninit()),
        })
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_cons {
        self.ring.as_mut_ptr()
    }

    pub fn init(self) -> Result<ConsumerRing, RingError> {
        let ring: Box<xsk_ring_cons> =
            unsafe { MaybeUninit::<xsk_ring_cons>::assume_init(*self.ring).into() };
        if ring.size != self.size {
            return Err(RingError::Initialize);
        }

        let ring_ptr = NonNull::new(Box::into_raw(ring)).ok_or(RingError::RingIsNull)?;

        Ok(ConsumerRing(ring_ptr))
    }
}

pub struct ProducerRingUninit {
    size: u32,
    ring: Box<MaybeUninit<xsk_ring_prod>>,
}

impl Default for ProducerRingUninit {
    fn default() -> Self {
        Self {
            size: XSK_RING_PROD__DEFAULT_NUM_DESCS,
            ring: Box::new(MaybeUninit::<xsk_ring_prod>::uninit()),
        }
    }
}

impl ProducerRingUninit {
    pub fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(size));
        }

        Ok(Self {
            size,
            ring: MaybeUninit::<xsk_ring_prod>::uninit().into(),
        })
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_prod {
        self.ring.as_mut_ptr()
    }

    pub fn init(self) -> Result<ProducerRing, RingError> {
        let ring: Box<xsk_ring_prod> =
            unsafe { MaybeUninit::<xsk_ring_prod>::assume_init(*self.ring).into() };
        if ring.size != self.size {
            return Err(RingError::Initialize);
        }

        let ring_ptr = NonNull::new(Box::into_raw(ring)).ok_or(RingError::RingIsNull)?;

        Ok(ProducerRing(ring_ptr))
    }
}

pub struct ConsumerRing(NonNull<xsk_ring_cons>);

impl std::fmt::Debug for ConsumerRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}

impl std::ops::Deref for ConsumerRing {
    type Target = xsk_ring_cons;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl ConsumerRing {
    pub fn complete_address(&self, index: u32) -> *const u64 {
        unsafe { xsk_ring_cons__comp_addr(self.0.as_ptr(), index) }
    }

    #[inline(always)]
    pub fn peek(&self, size: u32, index: &mut u32) -> u32 {
        unsafe { xsk_ring_cons__peek(self.0.as_ptr(), size, index) }
    }

    #[inline(always)]
    pub fn release(&self, size: u32) {
        unsafe { xsk_ring_cons__release(self.0.as_ptr(), size) }
    }

    #[inline(always)]
    pub fn rx_descriptor(&self, index: u32) -> *const xdp_desc {
        unsafe { xsk_ring_cons__rx_desc(self.0.as_ptr(), index) }
    }

    #[inline(always)]
    pub fn cancel(&self, size: u32) {
        unsafe { xsk_ring_cons__cancel(self.0.as_ptr(), size) }
    }
}

pub struct ProducerRing(NonNull<xsk_ring_prod>);

impl std::fmt::Debug for ProducerRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}

impl std::ops::Deref for ProducerRing {
    type Target = xsk_ring_prod;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl ProducerRing {
    #[inline(always)]
    pub fn fill_address(&self, index: u32) -> *mut u64 {
        unsafe { xsk_ring_prod__fill_addr(self.0.as_ptr(), index) }
    }

    #[inline(always)]
    pub fn needs_wakeup(&self) -> bool {
        let value = unsafe { xsk_ring_prod__needs_wakeup(self.0.as_ptr()) };
        match value {
            0 => false,
            _other_values => true,
        }
    }

    #[inline(always)]
    pub fn reserve(&self, size: u32, index: &mut u32) -> u32 {
        unsafe { xsk_ring_prod__reserve(self.0.as_ptr(), size, index) }
    }

    #[inline(always)]
    pub fn submit(&self, size: u32) {
        unsafe { xsk_ring_prod__submit(self.0.as_ptr(), size) }
    }

    #[inline(always)]
    pub fn tx_descriptor(&self, index: u32) -> *mut xdp_desc {
        unsafe { xsk_ring_prod__tx_desc(self.0.as_ptr(), index) }
    }
}

pub struct DescriptorRing(Arc<ArrayQueue<Descriptor>>);

impl Default for DescriptorRing {
    fn default() -> Self {
        let size = XSK_RING_CONS__DEFAULT_NUM_DESCS * 2;
        let ring = ArrayQueue::<Descriptor>::new(size as usize);

        Self(Arc::new(ring))
    }
}

pub enum RingError {
    Size(u32),
    Initialize,
    RingIsNull,
}

impl std::fmt::Debug for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Size(ring_size) => {
                write!(f, "The ring size ({}) is not the power of two.", ring_size)
            }
            Self::Initialize => write!(f, "Failed to initialize the ring."),
            Self::RingIsNull => write!(f, "The ring pointer returned null."),
        }
    }
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RingError {}
