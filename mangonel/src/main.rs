use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use mangonel::{frame::Descriptor, socket::SocketBuilder};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let running = running.clone();
        move || {
            running.store(false, Ordering::SeqCst);
        }
    })
    .unwrap();

    let interface_name = "enp5s0";
    let queue_id = 0;
    let config = SocketBuilder::default();
    let (mut receiver, mut sender) = config.build(interface_name, queue_id).unwrap();

    let mut receiver_buffer = VecDeque::<Descriptor>::with_capacity(32);
    while running.load(Ordering::SeqCst) {
        let n = receiver.rx_burst(&mut receiver_buffer);
        for _ in 0..n {
            let mut descriptor = receiver_buffer.pop_front().unwrap();
            println!("{:?}", descriptor.get_data());
        }
    }
}
