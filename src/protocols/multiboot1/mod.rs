pub mod boot_info;
pub mod constants;
pub mod header;

use constants::{BOOT_MAGIC, MAGIC, SEARCH_LIMIT};
use header::{Header, HEADER_SIZE};
use super::boot_protocol::BootProtocol;

pub struct Multiboot1;

impl BootProtocol for Multiboot1 {
    unsafe fn detect(kernel_buffer: *const u8, kernel_size: usize) -> bool {
        unsafe {
            let limit: usize = kernel_size.min(SEARCH_LIMIT);
            let mut offset: usize = 0;
            while offset + HEADER_SIZE <= limit {
                let header: &Header = &*(kernel_buffer.add(offset) as *const Header);
                if header.magic == MAGIC && header.is_valid() {
                    return true;
                }
                offset += 4;
            }
            false
        }
    }
}

pub unsafe fn jump(physical_entry: u32, boot_info: *const u8) -> ! {
    unsafe { crate::arch::x86_64::jump::jump_32bit(physical_entry, BOOT_MAGIC, boot_info as u32) }
}
