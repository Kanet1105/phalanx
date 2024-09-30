use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let running = running.clone();
        move || {
            running.store(false, Ordering::SeqCst);
        }
    })
    .unwrap();

    // let interface_name = "br0";
    // let queue_id = 0;
    // let config = SocketBuilder::default();
    // let (mut receiver, mut sender) = config.build(interface_name,
    // queue_id).unwrap();

    // let mut receiver_buffer = VecDeque::<Descriptor>::with_capacity(64);
    // let mut sender_buffer = VecDeque::<Descriptor>::with_capacity(64);
    // while running.load(Ordering::SeqCst) {
    //     let received = receiver.rx_burst(&mut receiver_buffer);
    //     if received > 0 {
    //         for _ in 0..received {
    //             let mut descriptor = receiver_buffer.pop_front().unwrap();
    //             descriptor.get_data();
    //             sender_buffer.push_back(descriptor);
    //         }

    //         sender.tx_burst(&mut sender_buffer);
    //     }
    // }
}
