use super::EfiStatus;

#[repr(C)]
pub struct EfiSimpleTextOutput {
    pub reset: usize,
    pub output_string:
        unsafe extern "efiapi" fn(*mut Self, *const u16) -> EfiStatus,
    pub test_string: usize,
    pub query_mode: usize,
    pub set_mode: usize,
    pub set_attribute: usize,
    pub clear_screen: usize,
    pub set_cursor_position: usize,
    pub enable_cursor: usize,
    pub mode: *mut u8,
}
