pub mod constants;

use crate::efi::guid::EfiGuid;
use crate::efi::EfiStatus;

#[repr(C)]
pub struct EfiFileProtocol {
    pub revision: u64,
    pub open: unsafe extern "efiapi" fn(
        *mut Self,
        *mut *mut Self,
        *const u16,
        u64,
        u64,
    ) -> EfiStatus,
    pub close: unsafe extern "efiapi" fn(*mut Self) -> EfiStatus,
    pub delete: usize,
    pub read: unsafe extern "efiapi" fn(*mut Self, *mut usize, *mut u8) -> EfiStatus,
    pub write: usize,
    pub get_position: usize,
    pub set_position: usize,
    pub get_info:
        unsafe extern "efiapi" fn(*mut Self, *const EfiGuid, *mut usize, *mut u8) -> EfiStatus,
    pub set_info: usize,
    pub flush: usize,
}
