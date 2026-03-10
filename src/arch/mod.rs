#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub trait Architecture {
    const PAGE_SIZE: u64;
    const PAGE_OFFSET_MASK: u64 = Self::PAGE_SIZE - 1;
}
