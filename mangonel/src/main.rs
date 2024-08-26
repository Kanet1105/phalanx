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

    let interface_name = "wlx94a67e7c18ac";
    let queue_id = 0;
    let mut config = SocketBuilder::default();
    config.descriptor_count = 4;
    let (mut receiver, mut sender) = config.build(interface_name, queue_id).unwrap();

    let mut buffer = VecDeque::<Descriptor>::with_capacity(32);
    while running.load(Ordering::SeqCst) {
        let received = receiver.rx_burst(&mut buffer);
        if received > 0 {
            for descriptor_index in 0..received {
                let descriptor = buffer.get(descriptor_index as usize).unwrap();
                println!("{}", descriptor.address());
            }

            sender.tx_burst(&mut buffer);
        }
    }
}
