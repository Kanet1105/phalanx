use mangonel::{mmap::Mmap, umem::Umem, util::*};

fn main() {
    let frame_size: u32 = 2048;
    let frame_headroom_size: u32 = 0;
    let descriptor_count: u32 = 4096;
    let completion_ring_size: u32 = 4096;
    let fill_ring_size: u32 = 4096;

    setrlimit();

    let mmap = Mmap::new(frame_size, frame_headroom_size, descriptor_count, false).unwrap();
    let mut frame_vec = mmap.populate();
    let umem = Umem::new(completion_ring_size, fill_ring_size, &mmap).unwrap();
    umem.populate(frame_size, frame_headroom_size, descriptor_count);
    println!("{:?}", umem.completion_ring());
    println!("{:?}", umem.fill_ring());
}
