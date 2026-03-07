#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const u8,
    pub memory_map_size: usize,
    pub memory_descriptor_size: usize,
    pub memory_descriptor_version: u32,
    pub framebuffer_base: u64,
    pub framebuffer_size: usize,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pixels_per_scan_line: u32,
    pub framebuffer_pixel_format: u32,
}
