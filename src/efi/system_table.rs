use super::boot_services::EfiBootServices;
use super::simple_text_output::EfiSimpleTextOutput;
use super::table_header::EfiTableHeader;
use super::EfiHandle;

#[repr(C)]
pub struct EfiSystemTable {
    pub hdr: EfiTableHeader,
    pub firmware_vendor: *mut u16,
    pub firmware_revision: u32,
    pub console_in_handle: EfiHandle,
    pub con_in: *mut u8,
    pub console_out_handle: EfiHandle,
    pub con_out: *mut EfiSimpleTextOutput,
    pub standard_error_handle: EfiHandle,
    pub std_err: *mut u8,
    pub runtime_services: *mut u8,
    pub boot_services: *mut EfiBootServices,
}
