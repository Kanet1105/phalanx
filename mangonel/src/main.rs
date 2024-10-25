use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use mangonel::{interface::Port, packet::Packet};
use mangonel_libxdp_rs::{descriptor::Descriptor, socket::SocketBuilder};

fn main() {
    let flag = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let flag = flag.clone();
        move || {
            flag.store(false, Ordering::SeqCst);
        }
    })
    .unwrap();

    // let port = Port::new("wan", "lan").unwrap();
    // let worker = thread::spawn({
    //     let port = port.clone();
    //     let flag = flag.clone();
    //     move || worker(flag, port)
    // });

    // worker.join().unwrap();

    let switch = thread::spawn({
        let flag = flag.clone();
        move || switch(flag)
    });

    switch.join().unwrap();
}

pub fn switch(flag: Arc<AtomicBool>) {
    let interface_name = "br0";
    let queue_id = 0;
    let config = SocketBuilder::default();
    let (mut receiver, mut sender) = config.build(interface_name, queue_id).unwrap();

    let mut descriptor_address_buffer = VecDeque::<u64>::with_capacity(64);
    let mut receiver_buffer = VecDeque::<Descriptor>::with_capacity(64);
    let mut sender_buffer = VecDeque::<Descriptor>::with_capacity(64);

    while flag.load(Ordering::SeqCst) {
        receiver.umem().fill(&mut descriptor_address_buffer);

        let received = receiver.rx_burst(&mut receiver_buffer);
        if received > 0 {
            for _ in 0..received {
                let mut descriptor = receiver_buffer.pop_front().unwrap();
                let packet: Packet = descriptor.get_data().into();
                println!("{:?}", packet.destination_mac());
                println!("{:?}", packet.source_mac());
                sender_buffer.push_back(descriptor);
            }

            sender.tx_burst(&mut sender_buffer);
        }

        sender.umem().complete(&mut descriptor_address_buffer);
    }
}

pub fn worker(flag: Arc<AtomicBool>, port: Port) {
    let interface_name = &port.wan().name;
    let queue_id = 0;
    let mut config = SocketBuilder::default();
    config.frame_headroom_size = 6 + 16;
    let (mut receiver, mut sender) = config.build(interface_name, queue_id).unwrap();

    let mut descriptor_address_buffer = VecDeque::<u64>::with_capacity(64);
    let mut receiver_buffer = VecDeque::<Descriptor>::with_capacity(64);
    let mut sender_buffer = VecDeque::<Descriptor>::with_capacity(64);

    while flag.load(Ordering::SeqCst) {
        receiver.umem().fill(&mut descriptor_address_buffer);

        let received = receiver.rx_burst(&mut receiver_buffer);
        if received > 0 {
            for _ in 0..received {
                let mut descriptor = receiver_buffer.pop_front().unwrap();
                let packet: Packet = descriptor.get_data().into();
                println!("{:?}", packet);
                sender_buffer.push_back(descriptor);
            }

            sender.tx_burst(&mut sender_buffer);
        }

        sender.umem().complete(&mut descriptor_address_buffer);
    }
}
