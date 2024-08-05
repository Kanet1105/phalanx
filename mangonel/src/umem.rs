use std::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use mangonel_libxdp_sys::{
    xsk_ring_cons, xsk_ring_prod, xsk_umem, xsk_umem__create, xsk_umem_config,
    XSK_UMEM__DEFAULT_FRAME_HEADROOM,
};

use crate::{
    mmap::{Mmap, MmapError},
    util,
};

#[allow(unused)]
pub struct Umem {
    mmap: Mmap,
    umem: NonNull<xsk_umem>,
    completion_ring: xsk_ring_cons,
    fill_ring: xsk_ring_prod,
    packet_size: usize,
    buffer_length: usize,
}

impl Umem {
    pub fn new(
        packet_size: usize,
        buffer_length: usize,
        completion_ring_size: u32,
        fill_ring_size: u32,
    ) -> Result<Self, UmemError> {
        let mmap = Mmap::new(packet_size, buffer_length).map_err(UmemError::Mmap)?;

        if !util::is_power_of_two(completion_ring_size) {
            return Err(UmemError::CompletionRingSize(completion_ring_size));
        }

        if !util::is_power_of_two(fill_ring_size) {
            return Err(UmemError::FillRingSize(fill_ring_size));
        }

        let mut umem: *mut xsk_umem = ptr::null_mut();
        let mut completion_ring = MaybeUninit::<xsk_ring_cons>::zeroed();
        let mut fill_ring = MaybeUninit::<xsk_ring_prod>::zeroed();
        let umem_config = xsk_umem_config {
            fill_size: fill_ring_size,
            comp_size: completion_ring_size,
            frame_size: packet_size as u32,
            frame_headroom: XSK_UMEM__DEFAULT_FRAME_HEADROOM,
            flags: 0,
        };

        let value = unsafe {
            xsk_umem__create(
                &mut umem,
                mmap.as_ptr(),
                mmap.length() as u64,
                fill_ring.as_mut_ptr(),
                completion_ring.as_mut_ptr(),
                &umem_config,
            )
        };

        if value.is_negative() {
            let error = std::io::Error::from_raw_os_error(-value);

            return Err(UmemError::Initialize(error));
        }

        Ok(Self {
            mmap,
            umem: NonNull::new(umem).ok_or(UmemError::UmemIsNull)?,
            completion_ring: unsafe { completion_ring.assume_init() },
            fill_ring: unsafe { fill_ring.assume_init() },
            packet_size,
            buffer_length,
        })
    }

    pub fn as_ptr(&self) -> *mut xsk_umem {
        self.umem.as_ptr()
    }

    pub fn offset(&self, count: isize) -> *mut c_void {
        unsafe { self.mmap.as_ptr().offset(count) }
    }

    pub fn packet_size(&self) -> usize {
        self.packet_size
    }

    pub fn buffer_length(&self) -> usize {
        self.buffer_length
    }
}

#[derive(Debug)]
pub enum UmemError {
    Mmap(MmapError),
    CompletionRingSize(u32),
    FillRingSize(u32),
    Initialize(std::io::Error),
    UmemIsNull,
}

impl std::fmt::Display for UmemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompletionRingSize(size) => write!(
                f,
                "The completion ring size ({}) must be the power of two",
                size
            ),
            Self::FillRingSize(size) => {
                write!(f, "The fill ring size ({}) must be thw power of two", size)
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for UmemError {}
