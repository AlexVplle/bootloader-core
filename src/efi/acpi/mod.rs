pub mod constants;

use core::ptr;

use constants::{EBDA_SEGMENT_PTR, REAL_MODE_SEGMENT_SHIFT, RSDP_MAX_ADDRESS, RSDP_REVISION_OFFSET, RSDP_V1_SIZE, RSDP_V2_REVISION, RSDP_V2_SIZE};
use super::boot_services::EfiBootServices;
use super::boot_services::constants::{EFI_ACPI_RECLAIM_MEMORY, EFI_ALLOCATE_MAX_ADDRESS};
use super::configuration_table::{ACPI_10_TABLE_GUID, ACPI_20_TABLE_GUID, EfiConfigurationTable};
use super::system_table::EfiSystemTable;
use super::EfiStatus;

pub unsafe fn setup_bios_compat(
    boot_services: *mut EfiBootServices,
    system_table: *mut EfiSystemTable,
) {
    unsafe {
        let n: usize = (*system_table).number_of_table_entries;
        let tables: *mut EfiConfigurationTable = (*system_table).configuration_table;
        let mut root_system_description_pointer: *mut u8 = ptr::null_mut();
        for i in 0..n {
            let entry: &EfiConfigurationTable = &*tables.add(i);
            if entry.vendor_guid == ACPI_20_TABLE_GUID || entry.vendor_guid == ACPI_10_TABLE_GUID {
                root_system_description_pointer = entry.vendor_table;
                break;
            }
        }
        if root_system_description_pointer.is_null() {
            return;
        }
        let revision: u8 = *root_system_description_pointer.add(RSDP_REVISION_OFFSET);
        let root_system_description_pointer_size: usize = if revision >= RSDP_V2_REVISION { RSDP_V2_SIZE } else { RSDP_V1_SIZE };

        let mut root_system_description_pointer_dest: u64 = RSDP_MAX_ADDRESS;
        let status: EfiStatus = ((*boot_services).allocate_pages)(
            EFI_ALLOCATE_MAX_ADDRESS,
            EFI_ACPI_RECLAIM_MEMORY,
            1,
            &mut root_system_description_pointer_dest,
        );
        if status != 0 {
            return;
        }
        ptr::copy_nonoverlapping(root_system_description_pointer, root_system_description_pointer_dest as *mut u8, root_system_description_pointer_size);
        *(EBDA_SEGMENT_PTR as *mut u16) = (root_system_description_pointer_dest >> REAL_MODE_SEGMENT_SHIFT) as u16;
    }
}
