pub const LINUX_BOOT_FLAG_MAGIC: u16 = 0xAA55;
pub const LINUX_HEADER_MAGIC: u32 = 0x53726448;
pub const HEADER_BOOT_FLAG: usize = 0x1FE;
pub const HEADER_MAGIC: usize = 0x202;
pub const HEADER_MIN_SIZE: usize = 0x270;

pub const HEADER_SETUP_SECTS: usize = 0x1F1;
pub const HEADER_TYPE_OF_LOADER: usize = 0x210;
pub const HEADER_LOADFLAGS: usize = 0x211;
pub const HEADER_RAMDISK_IMAGE: usize = 0x218;
pub const HEADER_RAMDISK_SIZE: usize = 0x21C;
pub const HEADER_HEAP_END_PTR: usize = 0x224;
pub const HEADER_CMD_LINE_PTR: usize = 0x228;
pub const HEADER_INIT_SIZE: usize = 0x260;
pub const HEADER_HANDOVER_OFFSET: usize = 0x264;
pub const HEADER_COPY_SIZE: usize = 0x7F;

pub const SI_ORIG_VIDEO_IS_VGA: usize = 0x0C;
pub const SI_LFB_WIDTH: usize = 0x0F;
pub const SI_LFB_HEIGHT: usize = 0x11;
pub const SI_LFB_DEPTH: usize = 0x13;
pub const SI_LFB_BASE: usize = 0x15;
pub const SI_LFB_SIZE: usize = 0x19;
pub const SI_LFB_LINELENGTH: usize = 0x21;
pub const SI_RED_SIZE: usize = 0x23;
pub const SI_RED_POS: usize = 0x24;
pub const SI_GREEN_SIZE: usize = 0x25;
pub const SI_GREEN_POS: usize = 0x26;
pub const SI_BLUE_SIZE: usize = 0x27;
pub const SI_BLUE_POS: usize = 0x28;
pub const SI_EXT_LFB_BASE: usize = 0x37;

pub const VIDEO_TYPE_EFI: u8 = 0x70;
pub const LOADFLAGS_LOADED_HIGH: u8 = 1 << 0;
pub const LOADFLAGS_CAN_USE_HEAP: u8 = 1 << 7;
