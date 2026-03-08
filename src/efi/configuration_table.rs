use super::guid::EfiGuid;

#[repr(C)]
pub struct EfiConfigurationTable {
    pub vendor_guid: EfiGuid,
    pub vendor_table: *mut u8,
}

pub const ACPI_10_TABLE_GUID: EfiGuid = EfiGuid {
    data1: 0xEB9D2D30,
    data2: 0x2D88,
    data3: 0x11D3,
    data4: [0x9A, 0x16, 0x00, 0x90, 0x27, 0x3F, 0xC1, 0x4D],
};

pub const ACPI_20_TABLE_GUID: EfiGuid = EfiGuid {
    data1: 0x8868E871,
    data2: 0xE4F1,
    data3: 0x11D3,
    data4: [0xBC, 0x22, 0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81],
};
