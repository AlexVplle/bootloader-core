use core::ptr;
use crate::efi::boot_services::{EFI_ALLOCATE_ANY_PAGES, EFI_LOADER_DATA, EfiBootServices};
use crate::helpers::check;
use crate::segment_mapping::SegmentMapping;

const PRESENT: u64 = 1;
const WRITABLE: u64 = 1 << 1;
const HUGE: u64 = 1 << 7;
const PAGE_TABLE_INDEX_MASK: u64 = 0x1FF;

unsafe fn alloc_page(boot_services: *mut EfiBootServices) -> u64 {
    unsafe {
        let mut page_physical_address: u64 = 0;
        check(((*boot_services).allocate_pages)(
            EFI_ALLOCATE_ANY_PAGES,
            EFI_LOADER_DATA,
            1,
            &mut page_physical_address,
        ));
        ptr::write_bytes(page_physical_address as *mut u8, 0, 0x1000);
        page_physical_address
    }
}

unsafe fn entry_ptr(page_table_physical_address: u64, index: usize) -> *mut u64 {
    unsafe { (page_table_physical_address as *mut u64).add(index) }
}

unsafe fn ensure_table(boot_services: *mut EfiBootServices, entry: *mut u64) -> u64 {
    unsafe {
        if *entry & PRESENT == 0 {
            let page_physical_address: u64 = alloc_page(boot_services);
            *entry = page_physical_address | PRESENT | WRITABLE;
        }
        *entry & !0xFFF
    }
}

unsafe fn map_4kb(
    boot_services: *mut EfiBootServices,
    page_map_level_4: u64,
    virtual_address: u64,
    physical_address: u64,
) {
    unsafe {
        let page_map_level_4_i: usize = ((virtual_address >> 39) & PAGE_TABLE_INDEX_MASK) as usize;
        let page_directory_pointer_table_i: usize = ((virtual_address >> 30) & PAGE_TABLE_INDEX_MASK) as usize;
        let page_directory_i: usize = ((virtual_address >> 21) & PAGE_TABLE_INDEX_MASK) as usize;
        let page_table_i: usize = ((virtual_address >> 12) & PAGE_TABLE_INDEX_MASK) as usize;

        let page_directory_pointer_table: u64 = ensure_table(boot_services, entry_ptr(page_map_level_4, page_map_level_4_i));
        let page_directory: u64 = ensure_table(boot_services, entry_ptr(page_directory_pointer_table, page_directory_pointer_table_i));

        let page_directory_entry: *mut u64 = entry_ptr(page_directory, page_directory_i);
        if *page_directory_entry & HUGE != 0 {
            let base_2mb: u64 = *page_directory_entry & !0x1FFFFF_u64;
            let page_table: u64 = alloc_page(boot_services);
            for i in 0usize..512 {
                *entry_ptr(page_table, i) = (base_2mb + i as u64 * 0x1000) | PRESENT | WRITABLE;
            }
            *page_directory_entry = page_table | PRESENT | WRITABLE;
        }

        let page_table: u64 = ensure_table(boot_services, page_directory_entry);
        *entry_ptr(page_table, page_table_i) = physical_address | PRESENT | WRITABLE;
    }
}

pub unsafe fn build_page_tables(
    boot_services: *mut EfiBootServices,
    mappings: &[SegmentMapping],
) -> u64 {
    unsafe {
        let page_map_level_4: u64 = alloc_page(boot_services);

        let page_directory_pointer_table: u64 = alloc_page(boot_services);
        *entry_ptr(page_map_level_4, 0) = page_directory_pointer_table | PRESENT | WRITABLE;
        for page_directory_pointer_table_i in 0usize..4 {
            let page_directory: u64 = alloc_page(boot_services);
            *entry_ptr(page_directory_pointer_table, page_directory_pointer_table_i) = page_directory | PRESENT | WRITABLE;
            for page_directory_i in 0usize..512 {
                let physical_address: u64 = page_directory_pointer_table_i as u64 * 0x40000000 + page_directory_i as u64 * 0x200000;
                *entry_ptr(page_directory, page_directory_i) = physical_address | PRESENT | WRITABLE | HUGE;
            }
        }

        for mapping in mappings {
            for i in 0..mapping.pages {
                let virtual_address: u64 = mapping.virtual_page_base + i as u64 * 0x1000;
                let physical_address: u64 = mapping.physical_base_address + i as u64 * 0x1000;
                map_4kb(boot_services, page_map_level_4, virtual_address, physical_address);
            }
        }

        page_map_level_4
    }
}
