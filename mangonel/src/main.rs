use std::{collections::VecDeque, thread, time::Duration};

use mangonel::{packet::Packet, socket::Socket};

fn main() {
    let packet_size = 2048;
    let buffer_length = 4096;
    let queue_id = 0;
    let rx_ring_size = 4096;
    let tx_ring_size = 4096;
    let interface_name = "lo";

    let (mut receiver, mut sender) = Socket::new(
        packet_size,
        buffer_length,
        queue_id,
        rx_ring_size,
        tx_ring_size,
        interface_name,
    )
    .unwrap();

    let receiver_handle = thread::spawn(move || {
        let mut buffer = VecDeque::<Packet>::with_capacity(8);

        loop {
            let received = receiver.receive(&mut buffer);

            for index in 0..received {
                println!("{:?}", buffer[index as usize]);
            }
        }
    });

    receiver_handle.join().unwrap();
}
