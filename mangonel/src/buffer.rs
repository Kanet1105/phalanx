pub struct Packet<'a> {
    address: u64,
    length: u16,
    data: &'a mut [u8],
}

impl<'a> Packet<'a> {}
