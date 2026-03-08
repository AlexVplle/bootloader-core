#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub architecture: u32,
    pub header_length: u32,
    pub checksum: u32,
}

impl Header {
    pub fn is_valid(&self) -> bool {
        self.magic
            .wrapping_add(self.architecture)
            .wrapping_add(self.header_length)
            .wrapping_add(self.checksum)
            == 0
    }
}
