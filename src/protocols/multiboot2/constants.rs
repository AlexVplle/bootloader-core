pub const MAGIC: u32 = 0xE85250D6;
pub const BOOT_MAGIC: u32 = 0x36d76289;
pub const SEARCH_LIMIT: usize = 32768;

pub const TAG_COMMANDLINE: u32 = 1;
pub const TAG_BOOTLOADER_NAME: u32 = 2;
pub const TAG_MMAP: u32 = 6;
pub const TAG_FRAMEBUFFER: u32 = 8;
pub const TAG_EFI_MEMORY_MAP: u32 = 17;
pub const TAG_END: u32 = 0;

pub const FRAMEBUFFER_TYPE_RGB: u8 = 1;
