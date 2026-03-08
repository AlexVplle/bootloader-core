#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub flags: u32,
    pub checksum: u32,
}

impl Header {
    pub fn is_valid(&self) -> bool {
        self.magic.wrapping_add(self.flags).wrapping_add(self.checksum) == 0
    }
}

pub const HEADER_SIZE: usize = core::mem::size_of::<Header>();
