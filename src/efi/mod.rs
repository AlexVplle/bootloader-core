pub type EfiHandle = *mut u8;
pub type EfiStatus = usize;

pub const EFI_SUCCESS: EfiStatus = 0;

pub mod boot_services;
pub mod file_info;
pub mod file_protocol;
pub mod guid;
pub mod loaded_image;
pub mod simple_file_system;
pub mod gop;
pub mod simple_text_output;
pub mod system_table;
pub mod table_header;
pub mod time;
