#![no_main]
#![no_std]


mod allocator;
mod efi;
mod elf;
mod gdt;
mod helpers;
mod jump;
mod loader;
mod paging;
mod protocols;
mod segment_mapping;

use efi::boot_services::EfiBootServices;
use efi::boot_services::constants::{EFI_ALLOCATE_MAX_ADDRESS, EFI_LOADER_DATA};
use efi::boot_services::get_memory_map_and_exit;
use efi::acpi::setup_bios_compat;
use efi::protocol::gop::mode::EfiGraphicsOutputProtocolMode;
use efi::protocol::gop::get_gop_info;
use efi::system_table::EfiSystemTable;
use efi::{EfiHandle, EfiStatus};
use elf::loader::load_elf;
use helpers::{check, halt};
use efi::protocol::file_protocol::EfiFileProtocol;
use paging::constants::PAGE_SIZE;
use loader::{open_root_dir, read_kernel};
use paging::build_page_tables;
use protocols::boot_protocol::BootProtocol;
use protocols::multiboot1::boot_info::BootInfoBuilder as Multiboot1BootInfoBuilder;
use protocols::multiboot2::boot_info::BootInfoBuilder;
use segment_mapping::SegmentMapping;

enum Protocol {
    Linux,
    Limine,
    Multiboot2,
    Multiboot1,
}

unsafe fn detect_protocol(kernel_buffer: *const u8, kernel_size: usize) -> Option<Protocol> {
    if unsafe { protocols::linux::Linux::detect(kernel_buffer, kernel_size) } {
        Some(Protocol::Linux)
    } else if unsafe { protocols::limine::Limine::detect(kernel_buffer, kernel_size) } {
        Some(Protocol::Limine)
    } else if unsafe { protocols::multiboot2::Multiboot2::detect(kernel_buffer, kernel_size) } {
        Some(Protocol::Multiboot2)
    } else if unsafe { protocols::multiboot1::Multiboot1::detect(kernel_buffer, kernel_size) } {
        Some(Protocol::Multiboot1)
    } else {
        None
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    halt()
}


#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus {
    unsafe {
        let boot_services: *mut EfiBootServices = (*system_table).boot_services;
        check(((*boot_services).set_watchdog_timer)(
            0,
            0,
            0,
            core::ptr::null(),
        ));

        let root: *mut EfiFileProtocol = open_root_dir(boot_services, image_handle);
        let (kernel_buffer, kernel_size): (*mut u8, usize) = read_kernel(boot_services, root);
        ((*root).close)(root);

        let protocol: Protocol = match detect_protocol(kernel_buffer, kernel_size) {
            Some(p) => p,
            None => halt(),
        };

        if matches!(protocol, Protocol::Linux) {
            let gop_mode: *mut EfiGraphicsOutputProtocolMode = get_gop_info(boot_services);
            protocols::linux::boot(
                boot_services,
                image_handle,
                system_table,
                kernel_buffer,
                kernel_size,
                gop_mode,
            );
        }

        let mut mappings: [SegmentMapping; 16] = [SegmentMapping {
            virtual_page_base: 0,
            physical_base_address: 0,
            pages: 0,
        }; 16];
        let mut mapping_count: usize = 0;
        let entry: u64 = load_elf(
            boot_services,
            kernel_buffer,
            &mut mappings,
            &mut mapping_count,
        );
        let page_map_level_4: u64 = build_page_tables(boot_services, &mappings[..mapping_count]);
        let gop_mode: *mut EfiGraphicsOutputProtocolMode = get_gop_info(boot_services);

        setup_bios_compat(boot_services, system_table);

        let mut boot_info_buffer: u64 = 0x200000;
        check(((*boot_services).allocate_pages)(
            EFI_ALLOCATE_MAX_ADDRESS,
            EFI_LOADER_DATA,
            16,
            &mut boot_info_buffer,
        ));
        let boot_info_buffer: *mut u8 = boot_info_buffer as *mut u8;

        let (mmap_buffer, mmap_size, desc_size, desc_ver) =
            get_memory_map_and_exit(boot_services, image_handle);

        core::arch::asm!("mov cr3, {}", in(reg) page_map_level_4);

        match protocol {
            Protocol::Linux => halt(),
            Protocol::Multiboot1 => {
                let physical_entry: u32 = {
                    let mut physical_address: u64 = 0;
                    let mut i: usize = 0;
                    while i < mapping_count {
                        let m = &mappings[i];
                        if entry >= m.virtual_page_base
                            && entry < m.virtual_page_base + m.pages as u64 * PAGE_SIZE
                        {
                            physical_address =
                                m.physical_base_address + (entry - m.virtual_page_base);
                            break;
                        }
                        i += 1;
                    }
                    physical_address as u32
                };
                let (red_position, green_position, blue_position): (u8, u8, u8) =
                    match (*(*gop_mode).info).pixel_format {
                        0 => (0, 8, 16),
                        1 => (16, 8, 0),
                        _ => halt(),
                    };
                let mut builder: Multiboot1BootInfoBuilder =
                    Multiboot1BootInfoBuilder::new(boot_info_buffer, boot_info_buffer as u32);
                builder.add_cmdline(b"");
                builder.add_loader_name(b"bootloader_from_scratch");
                builder.add_mmap(mmap_buffer, mmap_size, desc_size);
                builder.add_framebuffer(
                    (*gop_mode).frame_buffer_base,
                    (*(*gop_mode).info).horizontal_resolution,
                    (*(*gop_mode).info).vertical_resolution,
                    (*(*gop_mode).info).pixels_per_scan_line,
                    red_position,
                    green_position,
                    blue_position,
                );
                let boot_info: *const u8 = builder.finish();
                protocols::multiboot1::jump(physical_entry, boot_info)
            }
            Protocol::Multiboot2 => {
                let physical_entry: u32 = {
                    let mut physical_address: u64 = 0;
                    let mut i: usize = 0;
                    while i < mapping_count {
                        let m = &mappings[i];
                        if entry >= m.virtual_page_base
                            && entry < m.virtual_page_base + m.pages as u64 * PAGE_SIZE
                        {
                            physical_address =
                                m.physical_base_address + (entry - m.virtual_page_base);
                            break;
                        }
                        i += 1;
                    }
                    physical_address as u32
                };
                let (red_position, green_position, blue_position): (u8, u8, u8) =
                    match (*(*gop_mode).info).pixel_format {
                        0 => (0, 8, 16),
                        1 => (16, 8, 0),
                        _ => halt(),
                    };
                let mut builder: BootInfoBuilder = BootInfoBuilder::new(boot_info_buffer);
                builder.add_commandline(b"");
                builder.add_bootloader_name(b"bootloader_from_scratch");
                builder.add_mmap(mmap_buffer, mmap_size, desc_size);
                builder.add_framebuffer(
                    (*gop_mode).frame_buffer_base,
                    (*(*gop_mode).info).horizontal_resolution,
                    (*(*gop_mode).info).vertical_resolution,
                    (*(*gop_mode).info).pixels_per_scan_line,
                    red_position,
                    green_position,
                    blue_position,
                );
                builder.add_efi_memory_map(mmap_buffer, mmap_size, desc_size, desc_ver);
                let boot_info: *const u8 = builder.finish();
                protocols::multiboot2::jump_32(physical_entry, boot_info)
            }
            Protocol::Limine => protocols::limine::boot(
                entry,
                boot_info_buffer,
                gop_mode,
                mmap_buffer,
                mmap_size,
                desc_size,
                &mappings[..mapping_count],
            ),
        }
    }
}
