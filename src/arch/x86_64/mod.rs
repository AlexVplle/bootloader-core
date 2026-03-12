pub mod gdt;
pub mod jump;
pub mod paging;

pub fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
