use std::{collections::VecDeque, thread, time::Duration};

use mangonel::socket::SocketBuilder;

fn main() {
    let interface_name = "enp5s0";

    let socket = SocketBuilder::default();
    println!("{:?}", socket);

    let socket = socket.build(interface_name, 0).unwrap();

    loop {}
}
