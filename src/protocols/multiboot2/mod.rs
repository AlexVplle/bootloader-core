pub mod boot_info;
pub mod constants;
pub mod header;

use constants::{BOOT_MAGIC, MAGIC, SEARCH_LIMIT};
use header::Header;
use super::boot_protocol::BootProtocol;

pub struct Multiboot2;

impl BootProtocol for Multiboot2 {
    unsafe fn detect(kernel_buffer: *const u8, kernel_size: usize) -> bool {
        unsafe {
            let limit: usize = kernel_size.min(SEARCH_LIMIT);
            let mut offset: usize = 0;
            while offset + core::mem::size_of::<Header>() <= limit {
                let header: &Header = &*(kernel_buffer.add(offset) as *const Header);
                if header.magic == MAGIC && header.is_valid() {
                    return true;
                }
                offset += 8;
            }
            false
        }
    }
}

pub unsafe fn jump_32(physical_entry: u32, boot_info: *const u8) -> ! {
    unsafe {
        crate::jump::jump_32bit(physical_entry, BOOT_MAGIC, boot_info as u32)
    }
}

