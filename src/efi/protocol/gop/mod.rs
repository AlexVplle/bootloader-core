pub mod mode;
pub mod mode_information;
pub mod protocol;

use crate::efi::boot_services::EfiBootServices;
use crate::efi::guid::EfiGuid;
use crate::helpers::check;
use mode::EfiGraphicsOutputProtocolMode;
use protocol::EfiGraphicsOutputProtocol;

pub const GOP_GUID: EfiGuid = EfiGuid {
    data1: 0x9042a9de,
    data2: 0x23dc,
    data3: 0x4a38,
    data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

pub unsafe fn get_gop_info(boot_services: *mut EfiBootServices) -> *mut EfiGraphicsOutputProtocolMode {
    unsafe {
        let mut gop: *mut EfiGraphicsOutputProtocol = core::ptr::null_mut();
        check(((*boot_services).locate_protocol)(
            &GOP_GUID,
            core::ptr::null_mut(),
            &mut gop as *mut *mut EfiGraphicsOutputProtocol as *mut *mut u8,
        ));
        (*gop).mode
    }
}
