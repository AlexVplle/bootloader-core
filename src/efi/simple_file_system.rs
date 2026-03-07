use super::file_protocol::EfiFileProtocol;
use super::guid::EfiGuid;
use super::EfiStatus;

pub static SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data1: 0x964E5B22,
    data2: 0x6459,
    data3: 0x11D2,
    data4: [0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
};

#[repr(C)]
pub struct EfiSimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume:
        unsafe extern "efiapi" fn(*mut Self, *mut *mut EfiFileProtocol) -> EfiStatus,
}
