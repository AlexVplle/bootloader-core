use super::guid::EfiGuid;
use super::EfiStatus;

pub const EFI_FILE_MODE_READ: u64 = 0x1;
pub const EFI_FILE_MODE_WRITE: u64 = 0x2;
pub const EFI_FILE_MODE_CREATE: u64 = 0x8000000000000000;

pub const EFI_FILE_READ_ONLY: u64 = 0x1;
pub const EFI_FILE_HIDDEN: u64 = 0x2;
pub const EFI_FILE_SYSTEM: u64 = 0x4;
pub const EFI_FILE_DIRECTORY: u64 = 0x10;

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
