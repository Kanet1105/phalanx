use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use mangonel_libxdp_rs::{descriptor::Descriptor, socket::SocketBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let flag = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let flag = flag.clone();
        move || {
            flag.store(false, Ordering::SeqCst);
        }
    })
    .unwrap();

    let (mut sender, mut receiver) = SocketBuilder::default().build("enp5s0", 0)?;

    let worker = thread::spawn({
        let flag = flag.clone();
        let mut buffer = Vec::<Descriptor>::with_capacity(10);

        move || {
            while flag.load(Ordering::SeqCst) {
                let n = receiver.read(&mut buffer, 10);
                if n > 0 {
                    println!("Read: {:?}", n);

                    let n = sender.write(&buffer[0..n as usize]);
                    if n > 0 {
                        println!("Wrote: {}", n);
                    }
                }
            }
        }
    });

    worker.join().unwrap();

    Ok(())
}
