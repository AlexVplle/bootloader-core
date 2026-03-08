use core::ptr;

use crate::efi::boot_services::EfiBootServices;
use crate::efi::boot_services::constants::{EFI_LOADER_DATA, EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL};
use crate::efi::protocol::file_info::{EfiFileInfo, FILE_INFO_GUID};
use crate::efi::protocol::file_protocol::EfiFileProtocol;
use crate::efi::protocol::file_protocol::constants::EFI_FILE_MODE_READ;
use crate::efi::protocol::loaded_image::{EfiLoadedImageProtocol, LOADED_IMAGE_PROTOCOL_GUID};
use crate::efi::protocol::simple_file_system::{EfiSimpleFileSystemProtocol, SIMPLE_FILE_SYSTEM_PROTOCOL_GUID};
use crate::efi::EfiHandle;
use crate::helpers::check;

static KERNEL_NAME: [u16; 7] = [
    b'k' as u16,
    b'e' as u16,
    b'r' as u16,
    b'n' as u16,
    b'e' as u16,
    b'l' as u16,
    0,
];

pub unsafe fn open_root_dir(
    boot_services: *mut EfiBootServices,
    image_handle: EfiHandle,
) -> *mut EfiFileProtocol {
    unsafe {
        let mut loaded_image: *mut EfiLoadedImageProtocol = ptr::null_mut();
        check(((*boot_services).open_protocol)(
            image_handle,
            &LOADED_IMAGE_PROTOCOL_GUID,
            &mut loaded_image as *mut *mut EfiLoadedImageProtocol as *mut *mut u8,
            image_handle,
            ptr::null_mut(),
            EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
        ));

        let device_handle: *mut u8 = (*loaded_image).device_handle;
        let mut fs: *mut EfiSimpleFileSystemProtocol = ptr::null_mut();
        check(((*boot_services).open_protocol)(
            device_handle,
            &SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
            &mut fs as *mut *mut EfiSimpleFileSystemProtocol as *mut *mut u8,
            image_handle,
            ptr::null_mut(),
            EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
        ));

        let mut root: *mut EfiFileProtocol = ptr::null_mut();
        check(((*fs).open_volume)(fs, &mut root));
        root
    }
}

pub unsafe fn read_kernel(
    boot_services: *mut EfiBootServices,
    root: *mut EfiFileProtocol,
) -> (*mut u8, usize) {
    unsafe {
        let mut kernel_file: *mut EfiFileProtocol = ptr::null_mut();
        check(((*root).open)(root, &mut kernel_file, KERNEL_NAME.as_ptr(), EFI_FILE_MODE_READ, 0));

        let mut info_buffer: [u8; core::mem::size_of::<EfiFileInfo>()] =
            [0u8; core::mem::size_of::<EfiFileInfo>()];
        let mut info_size: usize = info_buffer.len();
        check(((*kernel_file).get_info)(kernel_file, &FILE_INFO_GUID, &mut info_size, info_buffer.as_mut_ptr()));
        let file_size: usize = (*(info_buffer.as_ptr() as *const EfiFileInfo)).file_size as usize;

        let mut kernel_buffer: *mut u8 = ptr::null_mut();
        check(((*boot_services).allocate_pool)(EFI_LOADER_DATA, file_size, &mut kernel_buffer));
        let mut read_size: usize = file_size;
        check(((*kernel_file).read)(kernel_file, &mut read_size, kernel_buffer));
        ((*kernel_file).close)(kernel_file);

        (kernel_buffer, file_size)
    }
}
