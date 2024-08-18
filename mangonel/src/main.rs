use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use arraydeque::{ArrayDeque, Wrapping};
use mangonel::{mmap::Mmap, packet::Frame, socket::SocketBuilder, umem::Umem, util::setrlimit};

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
    config.fill_ring_size = 64;
    let mmap = Mmap::initialize(
        config.frame_size,
        config.frame_headroom_size,
        config.descriptor_count,
        false,
    )
    .unwrap();
    let mut buffer = mmap.initialize_descriptor_buffer();
    let umem = Umem::initialize(config.completion_ring_size, config.fill_ring_size, &mmap).unwrap();
    umem.fill_ring().fill(&mut buffer).unwrap();

    let interface_name = "enp5s0";
    let queue_id = 0;
    let (mut receiver, mut _sender) = config.build(interface_name, queue_id, &umem).unwrap();
    let mut receive_buffer: ArrayDeque<Frame, 128, Wrapping> = ArrayDeque::new();

    let mut cnt = 0;
    while running.load(Ordering::SeqCst) {
        let n = receiver.rx_burst(&mut receive_buffer, &mmap);
        if n > 0 {
            // println!("{:?}", receiver.rx_ring());
        }
    }
}
