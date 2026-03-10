use crate::arch::x86_64::paging::constants::PAGE_SIZE;

#[derive(Copy, Clone)]
pub struct SegmentMapping {
    pub virtual_page_base: u64,
    pub physical_base_address: u64,
    pub pages: usize,
}

pub fn virtual_to_physical(virtual_address: u64, mappings: &[SegmentMapping]) -> u64 {
    for mapping in mappings {
        if virtual_address >= mapping.virtual_page_base
            && virtual_address < mapping.virtual_page_base + mapping.pages as u64 * PAGE_SIZE
        {
            return mapping.physical_base_address + (virtual_address - mapping.virtual_page_base);
        }
    }
    0
}
