pub type EfiHandle = *mut u8;
pub type EfiStatus = usize;

pub const EFI_SUCCESS: EfiStatus = 0;

pub mod acpi;
pub mod boot_services;
pub mod configuration_table;
pub mod guid;
pub mod protocol;
pub mod system_table;
pub mod table_header;
pub mod time;
