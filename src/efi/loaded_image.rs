use super::guid::EfiGuid;
use super::system_table::EfiSystemTable;
use super::EfiHandle;

pub static LOADED_IMAGE_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data1: 0x5B1B31A1,
    data2: 0x9562,
    data3: 0x11D2,
    data4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
};

#[repr(C)]
pub struct EfiLoadedImageProtocol {
    pub revision: u32,
    pub parent_handle: EfiHandle,
    pub system_table: *mut EfiSystemTable,
    pub device_handle: EfiHandle,
    pub file_path: *mut u8,
    pub reserved: *mut u8,
    pub load_options_size: u32,
    pub load_options: *mut u8,
    pub image_base: *mut u8,
    pub image_size: u64,
    pub image_code_type: u32,
    pub image_data_type: u32,
    pub unload: usize,
}
