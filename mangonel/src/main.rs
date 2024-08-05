use std::thread;

fn main() {
    let (mut receiver, sender) =
        mangonel::socket::Socket::new(2048, 4096, 0, 4096, 4096, "enp5s0").unwrap();

    let mut buffer = Vec::<mangonel::socket::Packet>::with_capacity(8);
    let received = receiver.receive(&mut buffer);
    println!("{}", received);

    // let receiver_handle = thread::spawn(move || {
    //     let mut buffer = Vec::<mangonel::socket::Packet>::with_capacity(8);

    //     loop {
    //         let received = receiver.receive(&mut buffer);

    //         for index in 0..received {
    //             println!("{:?}", buffer[index as usize]);
    //         }
    //     }
    // });

    // receiver_handle.join().unwrap();
}
