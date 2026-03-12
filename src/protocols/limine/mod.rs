pub mod constants;
pub mod framebuffer;
pub mod memory_map;

use core::ptr;

use constants::{
    BASE_REVISION_MAGIC, COMMON_MAGIC, FRAMEBUFFER_ID, MEMMAP_ID, REQUEST_RESPONSE_OFFSET,
};
use framebuffer::framebuffer::LimineFramebuffer;
use framebuffer::response::LimineFramebufferResponse;
use memory_map::constants::{
    ACPI_NVS, ACPI_RECLAIMABLE, BOOTLOADER_RECLAIMABLE, RESERVED, USABLE,
};
use memory_map::entry::LimineMemoryMapEntry;
use memory_map::response::LimineMemoryMapResponse;

use crate::allocator::bump::BumpAllocator;
use crate::arch::PAGE_SIZE;
use crate::framebuffer::FramebufferInfo;
use crate::segment_mapping::SegmentMapping;

fn efi_type_to_limine(efi_type: u32) -> u64 {
    match efi_type {
        1 | 2 => BOOTLOADER_RECLAIMABLE,
        3 | 4 | 7 => USABLE,
        9 => ACPI_RECLAIMABLE,
        10 => ACPI_NVS,
        _ => RESERVED,
    }
}

unsafe fn fulfill_framebuffer(
    request_ptr: *mut u8,
    alloc: &mut BumpAllocator,
    fb_info: &FramebufferInfo,
) {
    unsafe {
        let fb: *mut LimineFramebuffer = alloc.alloc::<LimineFramebuffer>();
        let fb_ptr_array: *mut *mut LimineFramebuffer = alloc.alloc::<*mut LimineFramebuffer>();
        let response: *mut LimineFramebufferResponse = alloc.alloc::<LimineFramebufferResponse>();

        ptr::write(fb, LimineFramebuffer {
            address: fb_info.base,
            width: fb_info.width as u64,
            height: fb_info.height as u64,
            pitch: fb_info.pitch as u64,
            bpp: 32,
            memory_model: 1,
            red_mask_size: 8,
            red_mask_shift: fb_info.red_pos,
            green_mask_size: 8,
            green_mask_shift: fb_info.green_pos,
            blue_mask_size: 8,
            blue_mask_shift: fb_info.blue_pos,
            unused: [0; 7],
            edid_size: 0,
            edid: 0,
        });

        *fb_ptr_array = fb;

        ptr::write(response, LimineFramebufferResponse {
            revision: 0,
            framebuffer_count: 1,
            framebuffers: fb_ptr_array,
        });

        *(request_ptr.add(REQUEST_RESPONSE_OFFSET) as *mut u64) = response as u64;
    }
}

unsafe fn fulfill_memory_map(
    request_ptr: *mut u8,
    alloc: &mut BumpAllocator,
    mmap_buffer: *const u8,
    mmap_size: usize,
    desc_size: usize,
) {
    unsafe {
        let entry_count: usize = mmap_size / desc_size;
        let entries: *mut LimineMemoryMapEntry =
            alloc.alloc_slice::<LimineMemoryMapEntry>(entry_count);
        let entry_ptrs: *mut *mut LimineMemoryMapEntry =
            alloc.alloc_slice::<*mut LimineMemoryMapEntry>(entry_count);
        let response: *mut LimineMemoryMapResponse = alloc.alloc::<LimineMemoryMapResponse>();

        for i in 0..entry_count {
            let desc: *const u8 = mmap_buffer.add(i * desc_size);
            let efi_type: u32 = *(desc as *const u32);
            let physical_start: u64 = *(desc.add(8) as *const u64);
            let number_of_pages: u64 = *(desc.add(24) as *const u64);

            let entry: *mut LimineMemoryMapEntry = entries.add(i);
            ptr::write(entry, LimineMemoryMapEntry {
                base: physical_start,
                length: number_of_pages * PAGE_SIZE,
                entry_type: efi_type_to_limine(efi_type),
            });
            *entry_ptrs.add(i) = entry;
        }

        ptr::write(response, LimineMemoryMapResponse {
            revision: 0,
            entry_count: entry_count as u64,
            entries: entry_ptrs,
        });

        *(request_ptr.add(REQUEST_RESPONSE_OFFSET) as *mut u64) = response as u64;
    }
}

unsafe fn scan_and_fulfill_requests(
    mappings: &[SegmentMapping],
    alloc: &mut BumpAllocator,
    fb_info: &FramebufferInfo,
    mmap_buffer: *const u8,
    mmap_size: usize,
    desc_size: usize,
) {
    unsafe {
        for mapping in mappings {
            let start: *mut u8 = mapping.physical_base_address as *mut u8;
            let size: usize = mapping.pages * PAGE_SIZE as usize;
            let mut offset: usize = 0;

            while offset + REQUEST_RESPONSE_OFFSET + 8 <= size {
                let ptr: *mut u8 = start.add(offset);
                let id0: u64 = *(ptr as *const u64);
                if id0 == BASE_REVISION_MAGIC[0] {
                    let id1: u64 = *(ptr.add(8) as *const u64);
                    if id1 == BASE_REVISION_MAGIC[1] {
                        *(ptr.add(16) as *mut u64) = 0;
                    }
                }
                if id0 == COMMON_MAGIC[0] {
                    let id1: u64 = *(ptr.add(8) as *const u64);
                    if id1 == COMMON_MAGIC[1] {
                        let id2: u64 = *(ptr.add(16) as *const u64);
                        let id3: u64 = *(ptr.add(24) as *const u64);
                        if id2 == FRAMEBUFFER_ID[0] && id3 == FRAMEBUFFER_ID[1] {
                            fulfill_framebuffer(ptr, alloc, fb_info);
                        } else if id2 == MEMMAP_ID[0] && id3 == MEMMAP_ID[1] {
                            fulfill_memory_map(ptr, alloc, mmap_buffer, mmap_size, desc_size);
                        }
                    }
                }
                offset += 8;
            }
        }
    }
}

use super::boot_protocol::BootProtocol;

pub struct Limine;

impl BootProtocol for Limine {
    unsafe fn detect(kernel_buffer: *const u8, kernel_size: usize) -> bool {
        unsafe {
            let mut offset: usize = 0;
            while offset + 16 <= kernel_size {
                let a: u64 = *(kernel_buffer.add(offset) as *const u64);
                if a == BASE_REVISION_MAGIC[0] {
                    let b: u64 = *(kernel_buffer.add(offset + 8) as *const u64);
                    if b == BASE_REVISION_MAGIC[1] {
                        return true;
                    }
                }
                offset += 8;
            }
            false
        }
    }
}

pub unsafe fn boot(
    entry: u64,
    boot_info_buffer: *mut u8,
    fb_info: &FramebufferInfo,
    mmap_buffer: *const u8,
    mmap_size: usize,
    desc_size: usize,
    mappings: &[SegmentMapping],
) -> ! {
    unsafe {
        let mut alloc: BumpAllocator = BumpAllocator::new(boot_info_buffer);
        scan_and_fulfill_requests(
            mappings,
            &mut alloc,
            fb_info,
            mmap_buffer,
            mmap_size,
            desc_size,
        );
        crate::arch::x86_64::jump::jump_64bit(entry);
    }
}
