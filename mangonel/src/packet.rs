#[derive(Debug)]
pub struct Packet<'a> {
    pub address: u64,
    pub length: u32,
    pub data: &'a mut [u8],
}
