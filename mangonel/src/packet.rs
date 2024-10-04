#[derive(Debug)]
pub struct Packet<'a>(&'a mut [u8]);

impl<'a> From<&'a mut [u8]> for Packet<'a> {
    fn from(value: &'a mut [u8]) -> Self {
        Self(value)
    }
}

impl<'a> Packet<'a> {}
