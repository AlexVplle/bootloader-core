pub const PRESENT: u64 = 1;
pub const WRITABLE: u64 = 1 << 1;
pub const HUGE: u64 = 1 << 7;
pub const PAGE_TABLE_INDEX_MASK: u64 = 0x1FF;

pub const PML4_SHIFT: u32 = 39;
pub const PDPT_SHIFT: u32 = 30;
pub const PD_SHIFT: u32 = 21;
pub const PT_SHIFT: u32 = 12;

pub const HUGE_PAGE_OFFSET_MASK: u64 = 0x1FFFFF;
pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;
pub const PAGE_SIZE: u64 = 0x1000;
pub const PAGE_OFFSET_MASK: u64 = PAGE_SIZE - 1;
pub const HUGE_PAGE_SIZE: u64 = 0x200000;
pub const PDPT_SPAN: u64 = 0x40000000;
