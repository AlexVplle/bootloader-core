#[derive(Copy, Clone)]
pub struct SegmentMapping {
    pub virtual_page_base: u64,
    pub physical_base_address: u64,
    pub pages: usize,
}
