// use std::{mem::MaybeUninit, ptr::NonNull};

// use mangonel_libxdp_sys::{
//     xdp_desc, xsk_ring_cons, xsk_ring_cons__cancel, xsk_ring_cons__peek,
// xsk_ring_cons__release,     xsk_ring_cons__rx_desc, xsk_ring_prod,
// xsk_ring_prod__fill_addr, xsk_ring_prod__needs_wakeup,
//     xsk_ring_prod__reserve, xsk_ring_prod__submit, xsk_ring_prod__tx_desc,
// };

// use crate::util::is_power_of_two;

// pub struct ConsumerRing(NonNull<xsk_ring_cons>);

// impl std::fmt::Debug for ConsumerRing {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", unsafe { self.0.as_ref() })
//     }
// }

// impl From<xsk_ring_cons> for ConsumerRing {
//     /// # Safety
//     ///
//     /// It is safe to `unwrap()` because null-checking has been done in
//     /// uninitialized ring types before the conversion.
//     fn from(value: xsk_ring_cons) -> Self {
//         let ring_ptr = Box::into_raw(Box::new(value));
//         let ring = NonNull::new(ring_ptr).unwrap();

//         Self(ring)
//     }
// }

// impl ConsumerRing {
//     #[inline(always)]
//     pub fn as_ptr(&self) -> *mut xsk_ring_cons {
//         self.0.as_ptr()
//     }

//     #[inline(always)]
//     pub fn peek(&self, size: u32, index: &mut u32) -> u32 {
//         unsafe { xsk_ring_cons__peek(self.as_ptr(), size, index) }
//     }

//     #[inline(always)]
//     pub fn release(&self, size: u32) {
//         unsafe { xsk_ring_cons__release(self.as_ptr(), size) }
//     }

//     #[inline(always)]
//     pub fn rx_descriptor(&self, index: u32) -> *const xdp_desc {
//         unsafe { xsk_ring_cons__rx_desc(self.as_ptr(), index) }
//     }

//     #[inline(always)]
//     pub fn cancel(&self, size: u32) {
//         unsafe { xsk_ring_cons__cancel(self.as_ptr(), size) }
//     }
// }

// pub struct ProducerRing(NonNull<xsk_ring_prod>);

// impl std::fmt::Debug for ProducerRing {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", unsafe { self.0.as_ref() })
//     }
// }

// impl From<xsk_ring_prod> for ProducerRing {
//     /// # Safety
//     ///
//     /// It is safe to `unwrap()` because null-checking has been done in
//     /// uninitialized ring types before the conversion.
//     fn from(value: xsk_ring_prod) -> Self {
//         let ring_ptr = Box::into_raw(Box::new(value));
//         let ring = NonNull::new(ring_ptr).unwrap();

//         Self(ring)
//     }
// }

// impl ProducerRing {
//     #[inline(always)]
//     pub fn as_ptr(&self) -> *mut xsk_ring_prod {
//         self.0.as_ptr()
//     }

//     #[inline(always)]
//     pub fn fill_address(&self, index: u32) -> *mut u64 {
//         unsafe { xsk_ring_prod__fill_addr(self.as_ptr(), index) }
//     }

//     #[inline(always)]
//     pub fn needs_wakeup(&self) -> bool {
//         let value = unsafe { xsk_ring_prod__needs_wakeup(self.as_ptr()) };
//         match value {
//             0 => false,
//             _other_values => true,
//         }
//     }

//     #[inline(always)]
//     pub fn reserve(&self, size: u32, index: &mut u32) -> u32 {
//         unsafe { xsk_ring_prod__reserve(self.as_ptr(), size, index) }
//     }

//     #[inline(always)]
//     pub fn submit(&self, size: u32) {
//         unsafe { xsk_ring_prod__submit(self.as_ptr(), size) }
//     }

//     #[inline(always)]
//     pub fn tx_descriptor(&self, index: u32) -> *mut xdp_desc {
//         unsafe { xsk_ring_prod__tx_desc(self.as_ptr(), index) }
//     }
// }

// pub struct CompletionRingUninit(MaybeUninit<xsk_ring_cons>, u32);

// impl CompletionRingUninit {
//     pub(crate) fn uninit(size: u32) -> Result<Self, RingError> {
//         if !is_power_of_two(size) {
//             return Err(RingError::Size(RingType::CompletionRing, size));
//         }

//         Ok(Self(MaybeUninit::uninit(), size))
//     }

//     pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_cons {
//         self.0.as_mut_ptr()
//     }

//     pub fn initialize(self) -> Result<CompletionRing, RingError> {
//         let ring = unsafe { self.0.assume_init() };
//         if ring.size != self.1 {
//             return Err(RingError::Initialize(RingType::CompletionRing));
//         }

//         Ok(CompletionRing::from(ring))
//     }
// }

// pub struct FillRingUninit(MaybeUninit<xsk_ring_prod>, u32);

// impl FillRingUninit {
//     pub(crate) fn uninit(size: u32) -> Result<Self, RingError> {
//         if !is_power_of_two(size) {
//             return Err(RingError::Size(RingType::FillRing, size));
//         }

//         Ok(Self(MaybeUninit::uninit(), size))
//     }

//     pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_prod {
//         self.0.as_mut_ptr()
//     }

//     pub fn initialize(self) -> Result<FillRing, RingError> {
//         let ring = unsafe { self.0.assume_init() };
//         if ring.size != self.1 {
//             return Err(RingError::Initialize(RingType::FillRing));
//         }

//         Ok(FillRing::from(ring))
//     }
// }

// pub struct RxRingUninit(MaybeUninit<xsk_ring_cons>, u32);

// impl RxRingUninit {
//     pub(crate) fn uninit(size: u32) -> Result<Self, RingError> {
//         if !is_power_of_two(size) {
//             return Err(RingError::Size(RingType::RxRing, size));
//         }

//         Ok(Self(MaybeUninit::uninit(), size))
//     }

//     pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_cons {
//         self.0.as_mut_ptr()
//     }

//     pub fn initialize(self) -> Result<RxRing, RingError> {
//         let ring = unsafe { self.0.assume_init() };
//         if ring.size != self.1 {
//             return Err(RingError::Initialize(RingType::RxRing));
//         }

//         Ok(RxRing::from(ring))
//     }
// }

// pub struct TxRingUninit(MaybeUninit<xsk_ring_prod>, u32);

// impl TxRingUninit {
//     pub(crate) fn uninit(size: u32) -> Result<Self, RingError> {
//         if !is_power_of_two(size) {
//             return Err(RingError::Size(RingType::TxRing, size));
//         }

//         Ok(Self(MaybeUninit::uninit(), size))
//     }

//     pub fn as_mut_ptr(&mut self) -> *mut xsk_ring_prod {
//         self.0.as_mut_ptr()
//     }

//     pub fn initialize(self) -> Result<TxRing, RingError> {
//         let ring = unsafe { self.0.assume_init() };
//         if ring.size != self.1 {
//             return Err(RingError::Initialize(RingType::TxRing));
//         }

//         Ok(TxRing::from(ring))
//     }
// }

// pub struct CompletionRing(ConsumerRing);

// impl std::fmt::Debug for CompletionRing {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.0)
//     }
// }

// impl std::ops::Deref for CompletionRing {
//     type Target = ConsumerRing;

//     #[inline(always)]
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<xsk_ring_cons> for CompletionRing {
//     fn from(value: xsk_ring_cons) -> Self {
//         Self(ConsumerRing::from(value))
//     }
// }

// impl CompletionRing {
//     pub fn uninitialized(size: u32) -> Result<CompletionRingUninit,
// RingError> {         CompletionRingUninit::uninit(size)
//     }
// }

// pub struct FillRing(ProducerRing);

// impl std::fmt::Debug for FillRing {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.0)
//     }
// }

// impl std::ops::Deref for FillRing {
//     type Target = ProducerRing;

//     #[inline(always)]
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<xsk_ring_prod> for FillRing {
//     fn from(value: xsk_ring_prod) -> Self {
//         Self(ProducerRing::from(value))
//     }
// }

// impl FillRing {
//     pub fn uninitialized(size: u32) -> Result<FillRingUninit, RingError> {
//         FillRingUninit::uninit(size)
//     }
// }

// pub struct RxRing(ConsumerRing);

// impl std::fmt::Debug for RxRing {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.0)
//     }
// }

// impl std::ops::Deref for RxRing {
//     type Target = ConsumerRing;

//     #[inline(always)]
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<xsk_ring_cons> for RxRing {
//     fn from(value: xsk_ring_cons) -> Self {
//         Self(ConsumerRing::from(value))
//     }
// }

// impl RxRing {
//     pub fn uninitialized(size: u32) -> Result<RxRingUninit, RingError> {
//         RxRingUninit::uninit(size)
//     }
// }

// pub struct TxRing(ProducerRing);

// impl std::fmt::Debug for TxRing {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.0)
//     }
// }

// impl std::ops::Deref for TxRing {
//     type Target = ProducerRing;

//     #[inline(always)]
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<xsk_ring_prod> for TxRing {
//     fn from(value: xsk_ring_prod) -> Self {
//         Self(ProducerRing::from(value))
//     }
// }

// impl TxRing {
//     pub fn uninitialized(size: u32) -> Result<TxRingUninit, RingError> {
//         TxRingUninit::uninit(size)
//     }
// }

// pub enum RingType {
//     CompletionRing,
//     FillRing,
//     RxRing,
//     TxRing,
// }

// impl std::fmt::Debug for RingType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::CompletionRing => write!(f, "{}",
// stringify!(CompletionRing)),             Self::FillRing => write!(f, "{}",
// stringify!(FillRing)),             Self::RxRing => write!(f, "{}",
// stringify!(RxRing)),             Self::TxRing => write!(f, "{}",
// stringify!(TxRing)),         }
//     }
// }

// pub enum RingError {
//     Size(RingType, u32),
//     Initialize(RingType),
//     Populate,
// }

// impl std::fmt::Debug for RingError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Size(ring_type, ring_size) => write!(
//                 f,
//                 "{:?} size: {} is not the power of two.",
//                 ring_type, ring_size
//             ),
//             Self::Initialize(ring_type) => write!(f, "Failed to initialize
// {:?}.", ring_type),             Self::Populate => write!(f, "Failed to
// populate the fill ring."),         }
//     }
// }

// impl std::fmt::Display for RingError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl std::error::Error for RingError {}

use std::{mem::MaybeUninit, ptr::NonNull};

use mangonel_libxdp_sys::{
    xdp_desc, xsk_ring_cons, xsk_ring_cons__peek, xsk_ring_cons__release, xsk_ring_cons__rx_desc,
    xsk_ring_prod, xsk_ring_prod__fill_addr, xsk_ring_prod__needs_wakeup, xsk_ring_prod__reserve,
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

    pub fn as_ptr(&self) -> *mut xsk_ring_cons {
        self.0.as_ptr()
    }

    pub fn peek(&self, index: &mut u32) -> u32 {
        unsafe { xsk_ring_cons__peek(self.as_ptr(), self.size, index) }
    }

    pub fn release(&self, descriptor_count: u32) {
        unsafe { xsk_ring_cons__release(self.as_ptr(), descriptor_count) }
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

    pub fn needs_wakeup(&self) -> bool {
        let value = unsafe { xsk_ring_prod__needs_wakeup(self.as_ptr()) };
        match value {
            0 => false,
            _other_values => true,
        }
    }

    pub fn reserve(&self, size: u32, index: &mut u32) -> u32 {
        unsafe { xsk_ring_prod__reserve(self.as_ptr(), size, index) }
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

unsafe impl Send for RxRing {}

unsafe impl Sync for RxRing {}

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

    pub fn peek(&self, batch_size: u32, index: &mut u32) -> u32 {
        unsafe { xsk_ring_cons__peek(self.as_ptr(), batch_size, index) }
    }

    pub fn rx_descriptor(&self, index: u32) -> *const xdp_desc {
        unsafe { xsk_ring_cons__rx_desc(self.as_ptr(), index) }
    }

    pub fn release(&self, descriptor_count: u32) {
        unsafe { xsk_ring_cons__release(self.as_ptr(), descriptor_count) }
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

unsafe impl Send for TxRing {}

unsafe impl Sync for TxRing {}

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
            Self::Initialize(ring_type) => write!(
                f,
                "Failed to initialize
{:?}.",
                ring_type
            ),
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
