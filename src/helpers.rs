use core::arch::asm;

use crate::efi::{EFI_SUCCESS, EfiStatus};

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}

pub fn check(status: EfiStatus) {
    if status != EFI_SUCCESS {
        halt();
    }
}

pub fn ascii_to_utf16(src: &[u8], dst: &mut [u16]) -> usize {
    let mut i: usize = 0;
    for &byte in src {
        if byte == b'\n' || byte == b'\r' || byte == 0 {
            break;
        }
        if i + 1 >= dst.len() {
            break;
        }
        dst[i] = byte as u16;
        i += 1;
    }
    dst[i] = 0;
    i
}
