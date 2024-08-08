use std::ptr::NonNull;

use mangonel_libxdp_sys::{xsk_ring_cons, xsk_ring_prod};

use crate::util::is_power_of_two;

pub struct CompletionRing(NonNull<xsk_ring_cons>);

impl CompletionRing {
    pub fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::CompletionRing, size));
        }

        Ok(Self(NonNull::dangling()))
    }

    pub fn as_ptr(&self) -> *mut xsk_ring_cons {
        self.0.as_ptr()
    }
}

pub struct FillRing(NonNull<xsk_ring_prod>);

impl FillRing {
    pub fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::FillRing, size));
        }

        Ok(Self(NonNull::dangling()))
    }

    pub fn as_ptr(&self) -> *mut xsk_ring_prod {
        self.0.as_ptr()
    }
}

pub struct RxRing(NonNull<xsk_ring_cons>);

impl RxRing {
    pub fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::RxRing, size));
        }

        Ok(Self(NonNull::dangling()))
    }

    pub fn as_ptr(&self) -> *mut xsk_ring_cons {
        self.0.as_ptr()
    }
}

pub struct TxRing(NonNull<xsk_ring_prod>);

impl TxRing {
    pub fn new(size: u32) -> Result<Self, RingError> {
        if !is_power_of_two(size) {
            return Err(RingError::Size(RingType::TxRing, size));
        }

        Ok(Self(NonNull::dangling()))
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
}

impl std::fmt::Debug for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Size(ring_type, ring_size) => write!(
                f,
                "{:?} size: {} is not the power of two",
                ring_type, ring_size
            ),
        }
    }
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
