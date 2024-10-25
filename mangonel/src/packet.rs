#[derive(Debug)]
pub struct Packet<'a>(&'a mut [u8]);

impl<'a> From<&'a mut [u8]> for Packet<'a> {
    fn from(value: &'a mut [u8]) -> Self {
        Self(value)
    }
}

impl<'a> Packet<'a> {
    #[inline(always)]
    pub fn destination_mac(&self) -> &[u8] {
        &self.0[0..6]
    }

    #[inline(always)]
    pub fn source_mac(&self) -> &[u8] {
        &self.0[6..12]
    }

    #[inline(always)]
    pub fn set_destination_mac(&mut self, address: &[u8]) {
        self.0[0..6].clone_from_slice(address);
    }

    #[inline(always)]
    pub fn set_source_mac(&mut self, address: &[u8]) {
        self.0[6..12].clone_from_slice(address);
    }

    #[inline(always)]
    pub fn set_source_ip(&mut self) {}

    #[inline(always)]
    pub fn set_destination_ip(&mut self) {}
}
