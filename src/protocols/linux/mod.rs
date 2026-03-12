pub mod constants;

use core::ptr;

use constants::*;
use crate::arch::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::firmware::FirmwareInterface;
use crate::framebuffer::FramebufferInfo;
use crate::protocols::boot_protocol::BootProtocol;

pub struct Linux;

impl BootProtocol for Linux {
    unsafe fn detect(kernel_buffer: *const u8, kernel_size: usize) -> bool {
        unsafe {
            if kernel_size < HEADER_MIN_SIZE {
                return false;
            }
            let boot_flag: u16 =
                ptr::read_unaligned(kernel_buffer.add(HEADER_BOOT_FLAG) as *const u16);
            if boot_flag != LINUX_BOOT_FLAG_MAGIC {
                return false;
            }
            let magic: u32 = ptr::read_unaligned(kernel_buffer.add(HEADER_MAGIC) as *const u32);
            magic == LINUX_HEADER_MAGIC
        }
    }
}

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

pub unsafe fn prepare<F: FirmwareInterface>(
    firmware: &mut F,
    kernel_buffer: *mut u8,
    kernel_size: usize,
    boot_params: *mut u8,
    cmdline_addr: u32,
    fb_info: &FramebufferInfo,
) -> u64 {
    unsafe {
        let mut setup_sects: u8 = read_u8(kernel_buffer, HEADER_SETUP_SECTS);
        if setup_sects == 0 {
            setup_sects = 4;
        }
        let pm_offset: usize = (setup_sects as usize + 1) * 512;
        let pm_size: usize = kernel_size - pm_offset;

        let init_size: u32 = read_u32(kernel_buffer, HEADER_INIT_SIZE);
        let handover_offset: u32 = read_u32(kernel_buffer, HEADER_HANDOVER_OFFSET);

        let alloc_pages: usize =
            ((init_size as usize).max(pm_size) + PAGE_OFFSET_MASK as usize) / PAGE_SIZE as usize;

        let pm_base: u64 = firmware.allocate_pages(alloc_pages);
        ptr::write_bytes(pm_base as *mut u8, 0, alloc_pages * PAGE_SIZE as usize);
        ptr::copy_nonoverlapping(kernel_buffer.add(pm_offset), pm_base as *mut u8, pm_size);

        ptr::copy_nonoverlapping(
            kernel_buffer.add(HEADER_SETUP_SECTS),
            boot_params.add(HEADER_SETUP_SECTS),
            HEADER_COPY_SIZE,
        );

        write_u8(boot_params, HEADER_TYPE_OF_LOADER, 0xFF);
        write_u8(
            boot_params,
            HEADER_LOADFLAGS,
            read_u8(boot_params, HEADER_LOADFLAGS) | LOADFLAGS_LOADED_HIGH | LOADFLAGS_CAN_USE_HEAP,
        );
        write_u16(boot_params, HEADER_HEAP_END_PTR, 0xFE00);
        write_u32(boot_params, HEADER_CMD_LINE_PTR, cmdline_addr);
        write_u32(boot_params, HEADER_RAMDISK_IMAGE, 0);
        write_u32(boot_params, HEADER_RAMDISK_SIZE, 0);

        let fb_size: u32 = fb_info.pitch * fb_info.height;

        write_u8(boot_params, SI_ORIG_VIDEO_IS_VGA, VIDEO_TYPE_EFI);
        write_u16(boot_params, SI_LFB_WIDTH, fb_info.width as u16);
        write_u16(boot_params, SI_LFB_HEIGHT, fb_info.height as u16);
        write_u16(boot_params, SI_LFB_DEPTH, 32);
        write_u32(boot_params, SI_LFB_BASE, fb_info.base as u32);
        write_u32(boot_params, SI_LFB_SIZE, fb_size);
        write_u16(boot_params, SI_LFB_LINELENGTH, fb_info.pitch as u16);
        write_u8(boot_params, SI_RED_SIZE, 8);
        write_u8(boot_params, SI_RED_POS, fb_info.red_pos);
        write_u8(boot_params, SI_GREEN_SIZE, 8);
        write_u8(boot_params, SI_GREEN_POS, fb_info.green_pos);
        write_u8(boot_params, SI_BLUE_SIZE, 8);
        write_u8(boot_params, SI_BLUE_POS, fb_info.blue_pos);
        write_u32(boot_params, SI_EXT_LFB_BASE, (fb_info.base >> 32) as u32);

        firmware.free_buffer(kernel_buffer);

        pm_base + handover_offset as u64 + 512
    }
}
