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

