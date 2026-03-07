use super::mode::EfiGraphicsOutputProtocolMode;
use super::mode_information::EfiGraphicsOutputModeInformation;
use crate::efi::EfiStatus;

#[repr(C)]
pub struct EfiGraphicsOutputProtocol {
    pub query_mode: unsafe extern "efiapi" fn(
        *mut Self,
        u32,
        *mut usize,
        *mut *mut EfiGraphicsOutputModeInformation,
    ) -> EfiStatus,
    pub set_mode: unsafe extern "efiapi" fn(*mut Self, u32) -> EfiStatus,
    pub blt: usize,
    pub mode: *mut EfiGraphicsOutputProtocolMode,
}
