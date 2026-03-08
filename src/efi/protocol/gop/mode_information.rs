#[repr(C)]
pub struct EfiGraphicsOutputModeInformation {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: u32,
    pub pixel_bitmask: [u32; 4],
    pub pixels_per_scan_line: u32,
}
