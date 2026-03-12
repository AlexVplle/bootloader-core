#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub const PAGE_SIZE: u64 = 0x1000;
#[cfg(target_arch = "x86_64")]
pub const PAGE_OFFSET_MASK: u64 = PAGE_SIZE - 1;
