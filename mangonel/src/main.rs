use std::{
    collections::VecDeque,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use arraydeque::{ArrayDeque, Wrapping};
use mangonel::{mmap::Mmap, socket::SocketBuilder, umem::Umem, util::setrlimit};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let running = running.clone();
        move || {
            running.store(false, Ordering::SeqCst);
        }
    })
    .unwrap();

    setrlimit();

    let mut config = SocketBuilder::default();
    let umem = Umem::new(
        config.frame_size,
        config.headroom_size,
        config.descriptor_count,
        config.completion_ring_size,
        config.fill_ring_size,
        false,
    )
    .unwrap();

    let frames: Vec<Frame> = (0..config.descriptor_count)
        .map(|descriptor_index: u32| {
            let offset = descriptor_index * config.frame_size;
            let address = umem.mmap().offset(offset as isize) as *mut u8;

            Frame {
                address: address as u64,
                length: 0,
                data: unsafe {
                    std::slice::from_raw_parts_mut(address, config.frame_size as usize)
                },
            }
        })
        .collect();

    for i in frames {
        println!("{:?}", i);
    }

    println!("{:?}", umem.fill_ring());
    let mut index: u32 = 0;
    println!("{:?}", umem.fill_ring().reserve(16, &mut index));

    // config.completion_ring_size;
    // let interface_name = "enp5s0";
    // let queue_id = 0;
    // let (mut receiver, mut sender, frames) = config.build(interface_name,
    // queue_id).unwrap();

    while running.load(Ordering::SeqCst) {
        // let n = receiver.rx_burst(burst_size: u32);
    }
}

#[derive(Debug)]
struct Frame<'a> {
    pub address: u64,
    pub length: u32,
    pub data: &'a mut [u8],
}
