pub mod constants;

use core::ptr;
use crate::firmware::FirmwareInterface;
use crate::segment_mapping::SegmentMapping;
use constants::{
    HUGE, HUGE_PAGE_OFFSET_MASK, HUGE_PAGE_SIZE, PAGE_OFFSET_MASK, PAGE_SIZE,
    PAGE_TABLE_ENTRY_COUNT, PAGE_TABLE_INDEX_MASK, PD_SHIFT, PDPT_SHIFT, PDPT_SPAN,
    PML4_SHIFT, PT_SHIFT, PRESENT, WRITABLE,
};

unsafe fn alloc_page<F: FirmwareInterface>(firmware: &mut F) -> u64 {
    unsafe {
        let page_physical_address: u64 = firmware.allocate_pages(1);
        ptr::write_bytes(page_physical_address as *mut u8, 0, PAGE_SIZE as usize);
        page_physical_address
    }
}

unsafe fn entry_ptr(page_table_physical_address: u64, index: usize) -> *mut u64 {
    unsafe { (page_table_physical_address as *mut u64).add(index) }
}

unsafe fn ensure_table<F: FirmwareInterface>(firmware: &mut F, entry: *mut u64) -> u64 {
    unsafe {
        if *entry & PRESENT == 0 {
            let page_physical_address: u64 = alloc_page(firmware);
            *entry = page_physical_address | PRESENT | WRITABLE;
        }
        *entry & !PAGE_OFFSET_MASK
    }
}

unsafe fn map_4kb<F: FirmwareInterface>(
    firmware: &mut F,
    page_map_level_4: u64,
    virtual_address: u64,
    physical_address: u64,
) {
    unsafe {
        let page_map_level_4_i: usize =
            ((virtual_address >> PML4_SHIFT) & PAGE_TABLE_INDEX_MASK) as usize;
        let page_directory_pointer_table_i: usize =
            ((virtual_address >> PDPT_SHIFT) & PAGE_TABLE_INDEX_MASK) as usize;
        let page_directory_i: usize =
            ((virtual_address >> PD_SHIFT) & PAGE_TABLE_INDEX_MASK) as usize;
        let page_table_i: usize =
            ((virtual_address >> PT_SHIFT) & PAGE_TABLE_INDEX_MASK) as usize;

        let page_directory_pointer_table: u64 =
            ensure_table(firmware, entry_ptr(page_map_level_4, page_map_level_4_i));
        let page_directory: u64 = ensure_table(
            firmware,
            entry_ptr(page_directory_pointer_table, page_directory_pointer_table_i),
        );

        let page_directory_entry: *mut u64 = entry_ptr(page_directory, page_directory_i);
        if *page_directory_entry & HUGE != 0 {
            let base_2mb: u64 = *page_directory_entry & !HUGE_PAGE_OFFSET_MASK;
            let page_table: u64 = alloc_page(firmware);
            for i in 0usize..PAGE_TABLE_ENTRY_COUNT {
                *entry_ptr(page_table, i) = (base_2mb + i as u64 * PAGE_SIZE) | PRESENT | WRITABLE;
            }
            *page_directory_entry = page_table | PRESENT | WRITABLE;
        }

        let page_table: u64 = ensure_table(firmware, page_directory_entry);
        *entry_ptr(page_table, page_table_i) = physical_address | PRESENT | WRITABLE;
    }
}

pub unsafe fn build_page_tables<F: FirmwareInterface>(
    firmware: &mut F,
    mappings: &[SegmentMapping],
) -> u64 {
    unsafe {
        let page_map_level_4: u64 = alloc_page(firmware);

        let page_directory_pointer_table: u64 = alloc_page(firmware);
        *entry_ptr(page_map_level_4, 0) = page_directory_pointer_table | PRESENT | WRITABLE;
        for page_directory_pointer_table_i in 0usize..4 {
            let page_directory: u64 = alloc_page(firmware);
            *entry_ptr(page_directory_pointer_table, page_directory_pointer_table_i) =
                page_directory | PRESENT | WRITABLE;
            for page_directory_i in 0usize..PAGE_TABLE_ENTRY_COUNT {
                let physical_address: u64 =
                    page_directory_pointer_table_i as u64 * PDPT_SPAN
                    + page_directory_i as u64 * HUGE_PAGE_SIZE;
                *entry_ptr(page_directory, page_directory_i) =
                    physical_address | PRESENT | WRITABLE | HUGE;
            }
        }

        for mapping in mappings {
            for i in 0..mapping.pages {
                let virtual_address: u64 = mapping.virtual_page_base + i as u64 * PAGE_SIZE;
                let physical_address: u64 = mapping.physical_base_address + i as u64 * PAGE_SIZE;
                map_4kb(firmware, page_map_level_4, virtual_address, physical_address);
            }
        }

        page_map_level_4
    }
}
