use core::ptr;

use crate::arch::Architecture;
use super::constants::{
    FRAMEBUFFER_TYPE_RGB, TAG_BOOTLOADER_NAME, TAG_COMMANDLINE, TAG_END, TAG_EFI_MEMORY_MAP,
    TAG_FRAMEBUFFER, TAG_MMAP,
};
use crate::allocator::bump::BumpAllocator;

pub struct BootInfoBuilder {
    alloc: BumpAllocator,
}

impl BootInfoBuilder {
    pub unsafe fn new(buffer: *mut u8) -> Self {
        unsafe {
            ptr::write_bytes(buffer, 0, 8);
            let mut alloc: BumpAllocator = BumpAllocator::new(buffer);
            alloc.skip(8);
            Self { alloc }
        }
    }

    unsafe fn add_string_tag(&mut self, tag_type: u32, s: &[u8]) {
        unsafe {
            self.alloc.write(tag_type);
            self.alloc.write(8u32 + s.len() as u32 + 1);
            let dst: *mut u8 = self.alloc.ptr_at_offset(self.alloc.current_offset());
            ptr::copy_nonoverlapping(s.as_ptr(), dst, s.len());
            self.alloc.skip(s.len());
            self.alloc.write(0u8);
            self.alloc.align_to(8);
        }
    }

    pub unsafe fn add_commandline(&mut self, cmdline: &[u8]) {
        unsafe { self.add_string_tag(TAG_COMMANDLINE, cmdline) }
    }

    pub unsafe fn add_bootloader_name(&mut self, name: &[u8]) {
        unsafe { self.add_string_tag(TAG_BOOTLOADER_NAME, name) }
    }

    pub unsafe fn add_mmap<A: Architecture>(
        &mut self,
        mmap_buffer: *const u8,
        mmap_size: usize,
        descriptor_size: usize,
    ) {
        unsafe {
            let entry_count: usize = mmap_size / descriptor_size;
            let entry_size: u32 = 24;
            let tag_data_size: u32 = 4 + 4 + entry_count as u32 * entry_size;
            self.alloc.write(TAG_MMAP);
            self.alloc.write(8u32 + tag_data_size);
            self.alloc.write(entry_size);
            self.alloc.write(0u32);
            for i in 0..entry_count {
                let desc: *const u8 = mmap_buffer.add(i * descriptor_size);
                let efi_type: u32 = *(desc as *const u32);
                let base: u64 = *(desc.add(8) as *const u64);
                let pages: u64 = *(desc.add(24) as *const u64);
                let mb2_type: u32 = match efi_type {
                    1 | 2 | 3 | 4 | 7 => 1,
                    9 => 3,
                    10 => 4,
                    _ => 2,
                };
                self.alloc.write(base);
                self.alloc.write(pages * A::PAGE_SIZE);
                self.alloc.write(mb2_type);
                self.alloc.write(0u32);
            }
            self.alloc.align_to(8);
        }
    }

    pub unsafe fn add_framebuffer(
        &mut self,
        base: u64,
        width: u32,
        height: u32,
        pixels_per_scan_line: u32,
        red_position: u8,
        green_position: u8,
        blue_position: u8,
    ) {
        unsafe {
            let pitch: u32 = pixels_per_scan_line * 4;
            self.alloc.write(TAG_FRAMEBUFFER);
            self.alloc.write(31u32 + 6);
            self.alloc.write(base);
            self.alloc.write(pitch);
            self.alloc.write(width);
            self.alloc.write(height);
            self.alloc.write(32u8);
            self.alloc.write(FRAMEBUFFER_TYPE_RGB);
            self.alloc.write(0u16);
            self.alloc.write(red_position);
            self.alloc.write(8u8);
            self.alloc.write(green_position);
            self.alloc.write(8u8);
            self.alloc.write(blue_position);
            self.alloc.write(8u8);
            self.alloc.align_to(8);
        }
    }

    pub unsafe fn add_efi_memory_map(
        &mut self,
        memory_map: *const u8,
        memory_map_size: usize,
        descriptor_size: usize,
        descriptor_version: u32,
    ) {
        unsafe {
            self.alloc.write(TAG_EFI_MEMORY_MAP);
            self.alloc.write((8 + 4 + 4 + memory_map_size) as u32);
            self.alloc.write(descriptor_size as u32);
            self.alloc.write(descriptor_version);
            let dst: *mut u8 = self.alloc.ptr_at_offset(self.alloc.current_offset());
            ptr::copy_nonoverlapping(memory_map, dst, memory_map_size);
            self.alloc.skip(memory_map_size);
            self.alloc.align_to(8);
        }
    }

    pub unsafe fn finish(mut self) -> *const u8 {
        unsafe {
            self.alloc.write(TAG_END);
            self.alloc.write(8u32);
            let total_size: u32 = self.alloc.current_offset() as u32;
            ptr::write(self.alloc.ptr_at_offset(0) as *mut u32, total_size);
            self.alloc.ptr_at_offset(0) as *const u8
        }
    }
}
