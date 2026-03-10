#[repr(C, packed)]
pub struct Gdtr {
    pub limit: u16,
    pub base: u64,
}
