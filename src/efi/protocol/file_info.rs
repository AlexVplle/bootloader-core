use crate::efi::guid::EfiGuid;
use crate::efi::time::EfiTime;

pub static FILE_INFO_GUID: EfiGuid = EfiGuid {
    data1: 0x09576E92,
    data2: 0x6D3F,
    data3: 0x11D2,
    data4: [0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
};

#[repr(C)]
pub struct EfiFileInfo {
    pub size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: EfiTime,
    pub last_access_time: EfiTime,
    pub modification_time: EfiTime,
    pub attribute: u64,
    pub file_name: [u16; 32],
}
