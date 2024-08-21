use std::{mem::MaybeUninit, ptr::NonNull};

use mangonel_libxdp_sys::{
    xdp_desc, xsk_ring_cons, xsk_ring_cons__cancel, xsk_ring_cons__peek, xsk_ring_cons__release,
    xsk_ring_cons__rx_desc, xsk_ring_prod, xsk_ring_prod__fill_addr, xsk_ring_prod__needs_wakeup,
    xsk_ring_prod__reserve, xsk_ring_prod__submit, xsk_ring_prod__tx_desc,
};

use crate::util::is_power_of_two;

pub struct ConsumerRingUninit(MaybeUninit<xsk_ring_cons>);

impl ConsumerRingUninit {
    fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(size));
        }

        Ok(Self(MaybeUninit::<xsk_ring_cons>::uninit()))
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_cons {
        self.0.as_mut_ptr()
    }

    pub fn init(self, size: u32) -> Result<ConsumerRing, RingError> {
        let ring: Box<xsk_ring_cons> = unsafe { self.0.assume_init().into() };
        if ring.size != size {
            return Err(RingError::Initialize);
        }
        let ring_ptr = NonNull::new(Box::into_raw(ring)).ok_or(RingError::RingIsNull)?;

        Ok(ConsumerRing(ring_ptr))
    }
}

pub struct ProducerRingUninit(MaybeUninit<xsk_ring_prod>);

impl ProducerRingUninit {
    fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(size));
        }

        Ok(Self(MaybeUninit::<xsk_ring_prod>::uninit()))
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_prod {
        self.0.as_mut_ptr()
    }

    pub fn init(self, size: u32) -> Result<ProducerRing, RingError> {
        let ring: Box<xsk_ring_prod> = unsafe { self.0.assume_init().into() };
        if ring.size != size {
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
    pub fn uninit(size: u32) -> Result<ConsumerRingUninit, RingError> {
        ConsumerRingUninit::new(size)
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
    pub fn uninit(size: u32) -> Result<ProducerRingUninit, RingError> {
        ProducerRingUninit::new(size)
    }

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

pub enum RingError {
    Size(u32),
    Initialize,
    RingIsNull,
}

impl std::fmt::Debug for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Size(ring_size) => {
                write!(f, "The ring size: {} is not the power of two", ring_size)
            }
            Self::Initialize => write!(f, "Failed to initialize the ring"),
            Self::RingIsNull => write!(f, "Ring pointer is null"),
        }
    }
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RingError {}
