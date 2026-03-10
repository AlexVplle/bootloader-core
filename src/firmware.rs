pub trait FirmwareInterface {
    unsafe fn allocate_pages(&mut self, count: usize) -> u64;
    unsafe fn try_allocate_pages_at(&mut self, address: u64, count: usize) -> Option<u64>;
    unsafe fn free_buffer(&mut self, ptr: *mut u8);
}
