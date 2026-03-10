pub mod gdt;
pub mod jump;
pub mod paging;

pub struct X86_64;

impl crate::arch::Architecture for X86_64 {
    const PAGE_SIZE: u64 = 0x1000;
}
