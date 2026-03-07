#![no_main]
#![no_std]

use core::mem;
use core::ptr;

mod boot_info;
mod efi;
mod elf;
mod helpers;
mod paging;
mod segment_mapping;

use boot_info::BootInfo;
use efi::boot_services::{
    EFI_ALLOCATE_ANY_PAGES, EFI_LOADER_CODE, EFI_LOADER_DATA, EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    EfiBootServices,
};
use efi::file_info::{EfiFileInfo, FILE_INFO_GUID};
use efi::file_protocol::{EFI_FILE_MODE_READ, EfiFileProtocol};
use efi::loaded_image::{EfiLoadedImageProtocol, LOADED_IMAGE_PROTOCOL_GUID};
use efi::simple_file_system::{EfiSimpleFileSystemProtocol, SIMPLE_FILE_SYSTEM_PROTOCOL_GUID};
use efi::system_table::EfiSystemTable;
use efi::gop::GOP_GUID;
use efi::gop::mode::EfiGraphicsOutputProtocolMode;
use efi::gop::protocol::EfiGraphicsOutputProtocol;
use efi::{EfiHandle, EfiStatus};
use elf::elf32::header::Elf32Header;
use elf::elf32::phdr::Elf32Phdr;
use elf::elf64::header::Elf64Header;
use elf::elf64::phdr::{ELF_MAGIC, Elf64Phdr, PT_LOAD};
use elf::{EI_CLASS, ELFCLASS32, ELFCLASS64};
use helpers::{ascii_to_utf16, check, halt};
use paging::build_page_tables;
use segment_mapping::SegmentMapping;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    halt()
}

static CONFIG_NAME: [u16; 9] = [
    b'b' as u16,
    b'o' as u16,
    b'o' as u16,
    b't' as u16,
    b'.' as u16,
    b'c' as u16,
    b'f' as u16,
    b'g' as u16,
    0,
];

unsafe fn open_root_dir(
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

unsafe fn read_kernel(
    boot_services: *mut EfiBootServices,
    root: *mut EfiFileProtocol,
) -> *mut u8 {
    unsafe {
        let mut cfg_file: *mut EfiFileProtocol = ptr::null_mut();
        check(((*root).open)(
            root,
            &mut cfg_file,
            CONFIG_NAME.as_ptr(),
            EFI_FILE_MODE_READ,
            0,
        ));
        let mut cfg_buffer: [u8; 256] = [0u8; 256];
        let mut cfg_size: usize = cfg_buffer.len();
        check(((*cfg_file).read)(cfg_file, &mut cfg_size, cfg_buffer.as_mut_ptr()));
        ((*cfg_file).close)(cfg_file);

        let mut kernel_name_utf16: [u16; 256] = [0u16; 256];
        ascii_to_utf16(&cfg_buffer[..cfg_size], &mut kernel_name_utf16);

        let mut kernel_file: *mut EfiFileProtocol = ptr::null_mut();
        check(((*root).open)(
            root,
            &mut kernel_file,
            kernel_name_utf16.as_ptr(),
            EFI_FILE_MODE_READ,
            0,
        ));

        let mut info_buffer: [u8; _] = [0u8; mem::size_of::<EfiFileInfo>()];
        let mut info_size: usize = info_buffer.len();
        check(((*kernel_file).get_info)(
            kernel_file,
            &FILE_INFO_GUID,
            &mut info_size,
            info_buffer.as_mut_ptr(),
        ));
        let file_size: usize = (*(info_buffer.as_ptr() as *const EfiFileInfo)).file_size as usize;

        let mut kernel_buffer: *mut u8 = ptr::null_mut();
        check(((*boot_services).allocate_pool)(EFI_LOADER_DATA, file_size, &mut kernel_buffer));
        let mut read_size: usize = file_size;
        check(((*kernel_file).read)(kernel_file, &mut read_size, kernel_buffer));
        ((*kernel_file).close)(kernel_file);

        kernel_buffer
    }
}

unsafe fn load_elf(
    boot_services: *mut EfiBootServices,
    kernel_buffer: *mut u8,
    mappings: &mut [SegmentMapping; 16],
    mapping_count: &mut usize,
) -> u64 {
    unsafe {
        if *(kernel_buffer as *const u32) != ELF_MAGIC {
            halt();
        }
        let ei_class: u8 = *kernel_buffer.add(EI_CLASS);
        let (entry, phoff, phnum, phentsize): (u64, u64, u16, u16) = match ei_class {
            ELFCLASS64 => {
                let elf: &Elf64Header = &*(kernel_buffer as *const Elf64Header);
                (elf.e_entry, elf.e_phoff, elf.e_phnum, elf.e_phentsize)
            }
            ELFCLASS32 => {
                let elf: &Elf32Header = &*(kernel_buffer as *const Elf32Header);
                (elf.e_entry as u64, elf.e_phoff as u64, elf.e_phnum, elf.e_phentsize)
            }
            _ => halt(),
        };

        for i in 0..phnum as usize {
            let (p_type, p_offset, p_vaddr, p_filesz, p_memsz): (u32, u64, u64, u64, u64) =
                match ei_class {
                    ELFCLASS64 => {
                        let phdr: &Elf64Phdr = &*((kernel_buffer as usize
                            + phoff as usize
                            + i * phentsize as usize)
                            as *const Elf64Phdr);
                        (phdr.p_type, phdr.p_offset, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz)
                    }
                    _ => {
                        let phdr: &Elf32Phdr = &*((kernel_buffer as usize
                            + phoff as usize
                            + i * phentsize as usize)
                            as *const Elf32Phdr);
                        (
                            phdr.p_type,
                            phdr.p_offset as u64,
                            phdr.p_vaddr as u64,
                            phdr.p_filesz as u64,
                            phdr.p_memsz as u64,
                        )
                    }
                };
            if p_type != PT_LOAD || p_memsz == 0 {
                continue;
            }
            let virtual_page_base: u64 = p_vaddr & !0xFFF;
            let page_offset: u64 = p_vaddr - virtual_page_base;
            let pages: usize = ((page_offset + p_memsz) as usize + 0xFFF) / 0x1000;
            let mut physical_base_address: u64 = 0;
            check(((*boot_services).allocate_pages)(
                EFI_ALLOCATE_ANY_PAGES,
                EFI_LOADER_CODE,
                pages,
                &mut physical_base_address,
            ));
            ptr::write_bytes(physical_base_address as *mut u8, 0, pages * 0x1000);
            ptr::copy_nonoverlapping(
                kernel_buffer.add(p_offset as usize),
                (physical_base_address + page_offset) as *mut u8,
                p_filesz as usize,
            );
            mappings[*mapping_count] = SegmentMapping { virtual_page_base, physical_base_address, pages };
            *mapping_count += 1;
        }

        check(((*boot_services).free_pool)(kernel_buffer));
        entry
    }
}

unsafe fn get_gop_info(boot_services: *mut EfiBootServices) -> *mut EfiGraphicsOutputProtocolMode {
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

unsafe fn get_memory_map_and_exit(
    boot_services: *mut EfiBootServices,
    image_handle: EfiHandle,
) -> BootInfo {
    unsafe {
        let mmap_buffer_size: usize = 0x8000;
        let mut mmap_buffer: *mut u8 = ptr::null_mut();
        check(((*boot_services).allocate_pool)(EFI_LOADER_DATA, mmap_buffer_size, &mut mmap_buffer));
        let mut mmap_size: usize = mmap_buffer_size;
        let mut map_key: usize = 0;
        let mut desc_size: usize = 0;
        let mut desc_ver: u32 = 0;
        check(((*boot_services).get_memory_map)(
            &mut mmap_size,
            mmap_buffer,
            &mut map_key,
            &mut desc_size,
            &mut desc_ver,
        ));
        check(((*boot_services).exit_boot_services)(image_handle, map_key));
        BootInfo {
            memory_map: mmap_buffer,
            memory_map_size: mmap_size,
            memory_descriptor_size: desc_size,
            memory_descriptor_version: desc_ver,
            framebuffer_base: 0,
            framebuffer_size: 0,
            framebuffer_width: 0,
            framebuffer_height: 0,
            framebuffer_pixels_per_scan_line: 0,
            framebuffer_pixel_format: 0,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus {
    unsafe {
        let boot_services: *mut EfiBootServices = (*system_table).boot_services;
        check(((*boot_services).set_watchdog_timer)(0, 0, 0, core::ptr::null()));
        let root: *mut EfiFileProtocol = open_root_dir(boot_services, image_handle);
        let kernel_buffer: *mut u8 = read_kernel(boot_services, root);
        ((*root).close)(root);
        let mut mappings: [SegmentMapping; 16] =
            [SegmentMapping { virtual_page_base: 0, physical_base_address: 0, pages: 0 }; 16];
        let mut mapping_count: usize = 0;
        let entry: u64 = load_elf(boot_services, kernel_buffer, &mut mappings, &mut mapping_count);
        let page_map_level_4: u64 = build_page_tables(boot_services, &mappings[..mapping_count]);
        let gop_mode: *mut EfiGraphicsOutputProtocolMode = get_gop_info(boot_services);
        let mut boot_info: BootInfo = get_memory_map_and_exit(boot_services, image_handle);
        boot_info.framebuffer_base = (*gop_mode).frame_buffer_base;
        boot_info.framebuffer_size = (*gop_mode).frame_buffer_size;
        boot_info.framebuffer_width = (*(*gop_mode).info).horizontal_resolution;
        boot_info.framebuffer_height = (*(*gop_mode).info).vertical_resolution;
        boot_info.framebuffer_pixels_per_scan_line = (*(*gop_mode).info).pixels_per_scan_line;
        boot_info.framebuffer_pixel_format = (*(*gop_mode).info).pixel_format;
        core::arch::asm!("mov cr3, {}", in(reg) page_map_level_4);
        let kernel_entry: extern "sysv64" fn(*const BootInfo) -> ! = mem::transmute(entry as usize);
        kernel_entry(&boot_info)
    }
}
