pub trait BootProtocol {
    unsafe fn detect(kernel_buffer: *const u8, kernel_size: usize) -> bool;
}
