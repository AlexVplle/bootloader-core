pub mod constants;

use core::ptr;

use constants::*;
use super::boot_protocol::BootProtocol;

pub struct Linux;

impl BootProtocol for Linux {
    unsafe fn detect(kernel_buffer: *const u8, kernel_size: usize) -> bool {
        unsafe {
            if kernel_size < HEADER_MIN_SIZE {
                return false;
            }
            let boot_flag: u16 = ptr::read_unaligned(kernel_buffer.add(HEADER_BOOT_FLAG) as *const u16);
            if boot_flag != LINUX_BOOT_FLAG_MAGIC {
                return false;
            }
            let magic: u32 = ptr::read_unaligned(kernel_buffer.add(HEADER_MAGIC) as *const u32);
            magic == LINUX_HEADER_MAGIC
        }
    }
}
use crate::efi::boot_services::EfiBootServices;
use crate::efi::boot_services::constants::{
    EFI_ALLOCATE_ANY_PAGES, EFI_ALLOCATE_MAX_ADDRESS, EFI_LOADER_CODE, EFI_LOADER_DATA,
};
use crate::efi::protocol::gop::mode::EfiGraphicsOutputProtocolMode;
use crate::efi::system_table::EfiSystemTable;
use crate::efi::EfiHandle;
use crate::helpers::{check, halt};
use crate::paging::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};

unsafe fn read_u8(base: *const u8, offset: usize) -> u8 {
    unsafe { *base.add(offset) }
}

unsafe fn read_u32(base: *const u8, offset: usize) -> u32 {
    unsafe { ptr::read_unaligned(base.add(offset) as *const u32) }
}

unsafe fn write_u8(base: *mut u8, offset: usize, val: u8) {
    unsafe { *base.add(offset) = val; }
}

unsafe fn write_u16(base: *mut u8, offset: usize, val: u16) {
    unsafe { ptr::write_unaligned(base.add(offset) as *mut u16, val); }
}

unsafe fn write_u32(base: *mut u8, offset: usize, val: u32) {
    unsafe { ptr::write_unaligned(base.add(offset) as *mut u32, val); }
}

pub unsafe fn boot(
    boot_services: *mut EfiBootServices,
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
    kernel_buffer: *mut u8,
    kernel_size: usize,
    gop_mode: *mut EfiGraphicsOutputProtocolMode,
) -> ! {
    unsafe {
        let mut setup_sects: u8 = read_u8(kernel_buffer, HEADER_SETUP_SECTS);
        if setup_sects == 0 {
            setup_sects = 4;
        }
        let pm_offset: usize = (setup_sects as usize + 1) * 512;
        let pm_size: usize = kernel_size - pm_offset;

        let init_size: u32 = read_u32(kernel_buffer, HEADER_INIT_SIZE);
        let handover_offset: u32 = read_u32(kernel_buffer, HEADER_HANDOVER_OFFSET);

        let alloc_pages: usize = ((init_size as usize).max(pm_size) + PAGE_OFFSET_MASK as usize) / PAGE_SIZE as usize;

        let mut pm_base: u64 = 0;
        check(((*boot_services).allocate_pages)(
            EFI_ALLOCATE_ANY_PAGES,
            EFI_LOADER_CODE,
            alloc_pages,
            &mut pm_base,
        ));
        ptr::write_bytes(pm_base as *mut u8, 0, alloc_pages * PAGE_SIZE as usize);
        ptr::copy_nonoverlapping(kernel_buffer.add(pm_offset), pm_base as *mut u8, pm_size);

        let mut boot_params_addr: u64 = 0xFFFFFFFF;
        check(((*boot_services).allocate_pages)(
            EFI_ALLOCATE_MAX_ADDRESS,
            EFI_LOADER_DATA,
            1,
            &mut boot_params_addr,
        ));
        let boot_params: *mut u8 = boot_params_addr as *mut u8;
        ptr::write_bytes(boot_params, 0, PAGE_SIZE as usize);

        let mut cmdline_addr: u64 = 0xFFFFFFFF;
        check(((*boot_services).allocate_pages)(
            EFI_ALLOCATE_MAX_ADDRESS,
            EFI_LOADER_DATA,
            1,
            &mut cmdline_addr,
        ));
        let cmdline_str: &[u8] = b"console=ttyS0,115200 nokaslr\0";
        ptr::copy_nonoverlapping(cmdline_str.as_ptr(), cmdline_addr as *mut u8, cmdline_str.len());

        ptr::copy_nonoverlapping(
            kernel_buffer.add(HEADER_SETUP_SECTS),
            boot_params.add(HEADER_SETUP_SECTS),
            HEADER_COPY_SIZE,
        );

        write_u8(boot_params, HEADER_TYPE_OF_LOADER, 0xFF);
        write_u8(boot_params, HEADER_LOADFLAGS, read_u8(boot_params, HEADER_LOADFLAGS) | LOADFLAGS_LOADED_HIGH | LOADFLAGS_CAN_USE_HEAP);
        write_u16(boot_params, HEADER_HEAP_END_PTR, 0xFE00);
        write_u32(boot_params, HEADER_CMD_LINE_PTR, cmdline_addr as u32);
        write_u32(boot_params, HEADER_RAMDISK_IMAGE, 0);
        write_u32(boot_params, HEADER_RAMDISK_SIZE, 0);

        let info: *const _ = (*gop_mode).info;
        let fb_base: u64 = (*gop_mode).frame_buffer_base;
        let width: u32 = (*info).horizontal_resolution;
        let height: u32 = (*info).vertical_resolution;
        let pitch: u32 = (*info).pixels_per_scan_line * 4;
        let fb_size: u32 = pitch * height;

        let (red_size, red_pos, green_size, green_pos, blue_size, blue_pos): (u8, u8, u8, u8, u8, u8) =
            match (*info).pixel_format {
                0 => (8, 0, 8, 8, 8, 16),
                1 => (8, 16, 8, 8, 8, 0),
                _ => halt(),
            };

        write_u8(boot_params, SI_ORIG_VIDEO_IS_VGA, VIDEO_TYPE_EFI);
        write_u16(boot_params, SI_LFB_WIDTH, width as u16);
        write_u16(boot_params, SI_LFB_HEIGHT, height as u16);
        write_u16(boot_params, SI_LFB_DEPTH, 32);
        write_u32(boot_params, SI_LFB_BASE, fb_base as u32);
        write_u32(boot_params, SI_LFB_SIZE, fb_size);
        write_u16(boot_params, SI_LFB_LINELENGTH, pitch as u16);
        write_u8(boot_params, SI_RED_SIZE, red_size);
        write_u8(boot_params, SI_RED_POS, red_pos);
        write_u8(boot_params, SI_GREEN_SIZE, green_size);
        write_u8(boot_params, SI_GREEN_POS, green_pos);
        write_u8(boot_params, SI_BLUE_SIZE, blue_size);
        write_u8(boot_params, SI_BLUE_POS, blue_pos);
        write_u32(boot_params, SI_EXT_LFB_BASE, (fb_base >> 32) as u32);

        ((*boot_services).free_pool)(kernel_buffer);

        let entry: u64 = pm_base + handover_offset as u64 + 512;
        let handover: extern "sysv64" fn(EfiHandle, *mut EfiSystemTable, *mut u8) -> ! =
            core::mem::transmute(entry);
        handover(image_handle, system_table, boot_params);
    }
}
