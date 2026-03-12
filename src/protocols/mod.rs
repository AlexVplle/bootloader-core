pub mod boot_protocol;
pub mod limine;
pub mod linux;
#[cfg(target_arch = "x86_64")]
pub mod multiboot1;
#[cfg(target_arch = "x86_64")]
pub mod multiboot2;
