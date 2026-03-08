pub const MAGIC: u32 = 0x1BADB002;
pub const BOOT_MAGIC: u32 = 0x2BADB002;
pub const SEARCH_LIMIT: usize = 8192;

pub const FLAG_MEM: u32 = 1 << 0;
pub const FLAG_CMDLINE: u32 = 1 << 2;
pub const FLAG_MMAP: u32 = 1 << 6;
pub const FLAG_LOADER_NAME: u32 = 1 << 9;
pub const FLAG_FRAMEBUFFER: u32 = 1 << 12;

pub const MMAP_AVAILABLE: u32 = 1;
pub const MMAP_RESERVED: u32 = 2;
pub const MMAP_ACPI: u32 = 3;
pub const MMAP_ACPI_NVS: u32 = 4;

pub const FRAMEBUFFER_TYPE_RGB: u8 = 1;

pub const MBI_FLAGS: usize = 0;
pub const MBI_MEM_LOWER: usize = 4;
pub const MBI_MEM_UPPER: usize = 8;
pub const MBI_CMDLINE: usize = 16;
pub const MBI_MMAP_LENGTH: usize = 44;
pub const MBI_MMAP_ADDR: usize = 48;
pub const MBI_BOOT_LOADER_NAME: usize = 64;
pub const MBI_FB_ADDR: usize = 88;
pub const MBI_FB_PITCH: usize = 96;
pub const MBI_FB_WIDTH: usize = 100;
pub const MBI_FB_HEIGHT: usize = 104;
pub const MBI_FB_BPP: usize = 108;
pub const MBI_FB_TYPE: usize = 109;
pub const MBI_FB_RED_POS: usize = 111;
pub const MBI_FB_RED_SIZE: usize = 112;
pub const MBI_FB_GREEN_POS: usize = 113;
pub const MBI_FB_GREEN_SIZE: usize = 114;
pub const MBI_FB_BLUE_POS: usize = 115;
pub const MBI_FB_BLUE_SIZE: usize = 116;
pub const MBI_STRUCT_SIZE: usize = 120;
