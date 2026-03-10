use core::ptr;

use crate::arch::Architecture;
use super::constants::*;
use crate::allocator::bump::BumpAllocator;

pub struct BootInfoBuilder {
    alloc: BumpAllocator,
    buf_phys: u32,
}

impl BootInfoBuilder {
    pub unsafe fn new(buf: *mut u8, buf_phys: u32) -> Self {
        unsafe {
            ptr::write_bytes(buf, 0, 4096);
            let mut alloc: BumpAllocator = BumpAllocator::new(buf);
            alloc.skip(MBI_STRUCT_SIZE);
            Self { alloc, buf_phys }
        }
    }

    unsafe fn set_u8(&self, offset: usize, val: u8) {
        unsafe { *self.alloc.ptr_at_offset(offset) = val; }
    }

    unsafe fn set_u32(&self, offset: usize, val: u32) {
        unsafe { ptr::write_unaligned(self.alloc.ptr_at_offset(offset) as *mut u32, val); }
    }

    unsafe fn set_u64(&self, offset: usize, val: u64) {
        unsafe { ptr::write_unaligned(self.alloc.ptr_at_offset(offset) as *mut u64, val); }
    }

    unsafe fn add_flags(&self, flags: u32) {
        unsafe {
            let current: u32 = ptr::read_unaligned(self.alloc.ptr_at_offset(MBI_FLAGS) as *const u32);
            self.set_u32(MBI_FLAGS, current | flags);
        }
    }

    unsafe fn append_str(&mut self, s: &[u8]) -> u32 {
        unsafe {
            let offset: usize = self.alloc.current_offset();
            let dst: *mut u8 = self.alloc.ptr_at_offset(offset);
            ptr::copy_nonoverlapping(s.as_ptr(), dst, s.len());
            self.alloc.skip(s.len());
            self.alloc.write(0u8);
            self.buf_phys + offset as u32
        }
    }

    pub unsafe fn add_cmdline(&mut self, cmdline: &[u8]) {
        unsafe {
            let phys: u32 = self.append_str(cmdline);
            self.set_u32(MBI_CMDLINE, phys);
            self.add_flags(FLAG_CMDLINE);
        }
    }

    pub unsafe fn add_loader_name(&mut self, name: &[u8]) {
        unsafe {
            let phys: u32 = self.append_str(name);
            self.set_u32(MBI_BOOT_LOADER_NAME, phys);
            self.add_flags(FLAG_LOADER_NAME);
        }
    }

    pub unsafe fn add_mmap<A: Architecture>(
        &mut self,
        mmap_buffer: *const u8,
        mmap_size: usize,
        descriptor_size: usize,
    ) {
        unsafe {
            let entry_count: usize = mmap_size / descriptor_size;
            let mmap_start: usize = self.alloc.current_offset();
            let mut mem_lower: u32 = 0;
            let mut mem_upper: u32 = 0;

            for i in 0..entry_count {
                let desc: *const u8 = mmap_buffer.add(i * descriptor_size);
                let efi_type: u32 = ptr::read_unaligned(desc as *const u32);
                let base: u64 = ptr::read_unaligned(desc.add(8) as *const u64);
                let pages: u64 = ptr::read_unaligned(desc.add(24) as *const u64);
                let length: u64 = pages * A::PAGE_SIZE;

                let mb1_type: u32 = match efi_type {
                    1 | 2 | 3 | 4 | 7 => MMAP_AVAILABLE,
                    9 => MMAP_ACPI,
                    10 => MMAP_ACPI_NVS,
                    _ => MMAP_RESERVED,
                };

                if mb1_type == MMAP_AVAILABLE {
                    if base < 0x100000 {
                        mem_lower += (length / 1024) as u32;
                    } else {
                        mem_upper += (length / 1024) as u32;
                    }
                }

                self.alloc.write(20u32);
                self.alloc.write(base);
                self.alloc.write(length);
                self.alloc.write(mb1_type);
            }

            let mmap_length: u32 = (self.alloc.current_offset() - mmap_start) as u32;
            self.set_u32(MBI_MMAP_ADDR, self.buf_phys + mmap_start as u32);
            self.set_u32(MBI_MMAP_LENGTH, mmap_length);
            self.set_u32(MBI_MEM_LOWER, mem_lower);
            self.set_u32(MBI_MEM_UPPER, mem_upper);
            self.add_flags(FLAG_MMAP | FLAG_MEM);
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
            self.set_u64(MBI_FB_ADDR, base);
            self.set_u32(MBI_FB_PITCH, pitch);
            self.set_u32(MBI_FB_WIDTH, width);
            self.set_u32(MBI_FB_HEIGHT, height);
            self.set_u8(MBI_FB_BPP, 32);
            self.set_u8(MBI_FB_TYPE, FRAMEBUFFER_TYPE_RGB);
            self.set_u8(MBI_FB_RED_POS, red_position);
            self.set_u8(MBI_FB_RED_SIZE, 8);
            self.set_u8(MBI_FB_GREEN_POS, green_position);
            self.set_u8(MBI_FB_GREEN_SIZE, 8);
            self.set_u8(MBI_FB_BLUE_POS, blue_position);
            self.set_u8(MBI_FB_BLUE_SIZE, 8);
            self.add_flags(FLAG_FRAMEBUFFER);
        }
    }

    pub fn finish(self) -> *const u8 {
        unsafe { self.alloc.ptr_at_offset(0) as *const u8 }
    }
}
