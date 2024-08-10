use std::{mem::MaybeUninit, ptr::NonNull};

use mangonel_libxdp_sys::{
    xsk_ring_cons, xsk_ring_prod, xsk_ring_prod__fill_addr, xsk_ring_prod__reserve,
    xsk_ring_prod__submit,
};

use crate::util::is_power_of_two;

pub struct CompletionRingUninit(MaybeUninit<xsk_ring_cons>);

impl CompletionRingUninit {
    fn new() -> Self {
        Self(MaybeUninit::<xsk_ring_cons>::uninit())
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_cons {
        self.0.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const xsk_ring_cons {
        self.0.as_ptr()
    }

    pub fn initialize(self) -> Result<CompletionRing, RingError> {
        let ring_boxed: Box<xsk_ring_cons> = unsafe { self.0.assume_init().into() };
        let ring_ptr = Box::into_raw(ring_boxed);
        let ring = NonNull::new(ring_ptr).ok_or(RingError::Initialize(RingType::CompletionRing))?;

        Ok(CompletionRing(ring))
    }
}

pub struct CompletionRing(NonNull<xsk_ring_cons>);

impl std::fmt::Debug for CompletionRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}

impl std::ops::Deref for CompletionRing {
    type Target = xsk_ring_cons;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl CompletionRing {
    pub fn uninitialized(size: u32) -> Result<CompletionRingUninit, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::CompletionRing, size));
        }

        Ok(CompletionRingUninit::new())
    }

    pub fn as_ptr(&mut self) -> *mut xsk_ring_cons {
        self.0.as_ptr()
    }
}

pub struct FillRingUninit(MaybeUninit<xsk_ring_prod>);

impl FillRingUninit {
    fn new() -> Self {
        Self(MaybeUninit::<xsk_ring_prod>::uninit())
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_prod {
        self.0.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const xsk_ring_prod {
        self.0.as_ptr()
    }

    pub fn initialize(self) -> Result<FillRing, RingError> {
        let ring_boxed: Box<xsk_ring_prod> = unsafe { self.0.assume_init().into() };
        let ring_ptr = Box::into_raw(ring_boxed);
        let ring = NonNull::new(ring_ptr).ok_or(RingError::Initialize(RingType::FillRing))?;

        Ok(FillRing(ring))
    }
}

pub struct FillRing(NonNull<xsk_ring_prod>);

impl std::fmt::Debug for FillRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}

impl std::ops::Deref for FillRing {
    type Target = xsk_ring_prod;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl FillRing {
    pub fn uninitialized(size: u32) -> Result<FillRingUninit, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::FillRing, size));
        }

        Ok(FillRingUninit::new())
    }

    pub fn as_ptr(&self) -> *mut xsk_ring_prod {
        self.0.as_ptr()
    }

    pub fn populate(&self, frame_size: u32) -> Result<(), RingError> {
        let mut index: u32 = 0;

        let value = unsafe { xsk_ring_prod__reserve(self.as_ptr(), self.size, &mut index) };
        if value != self.size {
            return Err(RingError::Populate);
        }

        for i in 0..self.size {
            index += 1;
            unsafe {
                *xsk_ring_prod__fill_addr(self.as_ptr(), index) = (i * frame_size) as u64;
            }
        }

        unsafe { xsk_ring_prod__submit(self.as_ptr(), self.size) }

        Ok(())
    }
}

pub struct RxRingUninit(MaybeUninit<xsk_ring_cons>);

impl RxRingUninit {
    fn new() -> Self {
        Self(MaybeUninit::<xsk_ring_cons>::uninit())
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_cons {
        self.0.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const xsk_ring_cons {
        self.0.as_ptr()
    }

    pub fn initialize(self) -> Result<RxRing, RingError> {
        let ring_boxed: Box<xsk_ring_cons> = unsafe { self.0.assume_init().into() };
        let ring_ptr = Box::into_raw(ring_boxed);
        let ring = NonNull::new(ring_ptr).ok_or(RingError::Initialize(RingType::RxRing))?;

        Ok(RxRing(ring))
    }
}

pub struct RxRing(NonNull<xsk_ring_cons>);

impl std::fmt::Debug for RxRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}

impl std::ops::Deref for RxRing {
    type Target = xsk_ring_cons;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl RxRing {
    pub fn uninitialized(size: u32) -> Result<RxRingUninit, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::RxRing, size));
        }

        Ok(RxRingUninit::new())
    }

    pub fn as_ptr(&self) -> *mut xsk_ring_cons {
        self.0.as_ptr()
    }
}

pub struct TxRingUninit(MaybeUninit<xsk_ring_prod>);

impl TxRingUninit {
    fn new() -> Self {
        Self(MaybeUninit::<xsk_ring_prod>::uninit())
    }

    pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_prod {
        self.0.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const xsk_ring_prod {
        self.0.as_ptr()
    }

    pub fn initialize(self) -> Result<TxRing, RingError> {
        let ring_boxed: Box<xsk_ring_prod> = unsafe { self.0.assume_init().into() };
        let ring_ptr = Box::into_raw(ring_boxed);
        let ring = NonNull::new(ring_ptr).ok_or(RingError::Initialize(RingType::TxRing))?;

        Ok(TxRing(ring))
    }
}

pub struct TxRing(NonNull<xsk_ring_prod>);

impl std::fmt::Debug for TxRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}

impl std::ops::Deref for TxRing {
    type Target = xsk_ring_prod;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl TxRing {
    pub fn uninitialized(size: u32) -> Result<TxRingUninit, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::TxRing, size));
        }

        Ok(TxRingUninit::new())
    }

    pub fn as_ptr(&self) -> *mut xsk_ring_prod {
        self.0.as_ptr()
    }
}

pub enum RingType {
    CompletionRing,
    FillRing,
    RxRing,
    TxRing,
}

impl std::fmt::Debug for RingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompletionRing => write!(f, "{}", stringify!(CompletionRing)),
            Self::FillRing => write!(f, "{}", stringify!(FillRing)),
            Self::RxRing => write!(f, "{}", stringify!(RxRing)),
            Self::TxRing => write!(f, "{}", stringify!(TxRing)),
        }
    }
}

pub enum RingError {
    Size(RingType, u32),
    Initialize(RingType),
    Populate,
}

impl std::fmt::Debug for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Size(ring_type, ring_size) => write!(
                f,
                "{:?} size: {} is not the power of two.",
                ring_type, ring_size
            ),
            Self::Initialize(ring_type) => write!(f, "Failed to initialize {:?}.", ring_type),
            Self::Populate => write!(f, "Failed to populate the fill ring."),
        }
    }
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RingError {}
