use core::ptr;

use crate::elf::constants::{EI_CLASS, ELF_MAGIC, ELFCLASS32, ELFCLASS64};
use crate::elf::elf32::header::Elf32Header;
use crate::elf::elf32::program_header::Elf32ProgramHeader;
use crate::elf::elf64::constants::PT_LOAD;
use crate::elf::elf64::header::Elf64Header;
use crate::elf::elf64::program_header::Elf64ProgramHeader;
use crate::firmware::FirmwareInterface;
use crate::helpers::halt;
use crate::arch::x86_64::paging::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::segment_mapping::{SegmentMapping, virtual_to_physical};

const SHT_RELA: u32 = 4;

#[cfg(target_arch = "x86_64")]
const ELF_R_RELATIVE: u32 = 8;
#[cfg(target_arch = "aarch64")]
const ELF_R_RELATIVE: u32 = 1027;
#[cfg(target_arch = "riscv64")]
const ELF_R_RELATIVE: u32 = 3;

unsafe fn load_segments<F: FirmwareInterface>(
    firmware: &mut F,
    kernel_buffer: *mut u8,
    ei_class: u8,
    mappings: &mut [SegmentMapping; 16],
    mapping_count: &mut usize,
) {
    unsafe {
        let (phoff, phnum, phentsize): (u64, u16, u16) = match ei_class {
            ELFCLASS64 => {
                let elf: &Elf64Header = &*(kernel_buffer as *const Elf64Header);
                (elf.e_phoff, elf.e_phnum, elf.e_phentsize)
            }
            _ => {
                let elf: &Elf32Header = &*(kernel_buffer as *const Elf32Header);
                (elf.e_phoff as u64, elf.e_phnum, elf.e_phentsize)
            }
        };

        for i in 0..phnum as usize {
            let (p_type, p_offset, p_vaddr, p_paddr, p_filesz, p_memsz): (u32, u64, u64, u64, u64, u64) =
                match ei_class {
                    ELFCLASS64 => {
                        let phdr: &Elf64ProgramHeader = &*((kernel_buffer as usize + phoff as usize + i * phentsize as usize) as *const Elf64ProgramHeader);
                        (phdr.p_type, phdr.p_offset, phdr.p_vaddr, phdr.p_paddr, phdr.p_filesz, phdr.p_memsz)
                    }
                    _ => {
                        let phdr: &Elf32ProgramHeader = &*((kernel_buffer as usize + phoff as usize + i * phentsize as usize) as *const Elf32ProgramHeader);
                        (phdr.p_type, phdr.p_offset as u64, phdr.p_vaddr as u64, phdr.p_paddr as u64, phdr.p_filesz as u64, phdr.p_memsz as u64)
                    }
                };
            if p_type != PT_LOAD || p_memsz == 0 {
                continue;
            }
            let virtual_page_base: u64 = p_vaddr & !PAGE_OFFSET_MASK;
            let page_offset: u64 = p_vaddr - virtual_page_base;
            let pages: usize = ((page_offset + p_memsz) as usize + PAGE_OFFSET_MASK as usize) / PAGE_SIZE as usize;

            let mut found_j: usize = usize::MAX;
            let mut j: usize = 0;
            while j < *mapping_count {
                let m: &SegmentMapping = &mappings[j];
                if virtual_page_base >= m.virtual_page_base
                    && virtual_page_base < m.virtual_page_base + m.pages as u64 * PAGE_SIZE
                {
                    found_j = j;
                    break;
                }
                j += 1;
            }

            let dest_physical: u64 = if found_j != usize::MAX {
                let existing_phys: u64 = mappings[found_j].physical_base_address
                    + (virtual_page_base - mappings[found_j].virtual_page_base);
                let segment_end: u64 = virtual_page_base + pages as u64 * PAGE_SIZE;
                let existing_end: u64 = mappings[found_j].virtual_page_base
                    + mappings[found_j].pages as u64 * PAGE_SIZE;
                if segment_end > existing_end {
                    let additional: usize = ((segment_end - existing_end) / PAGE_SIZE) as usize;
                    let new_phys: u64 = mappings[found_j].physical_base_address
                        + (existing_end - mappings[found_j].virtual_page_base);
                    let allocated: u64 = firmware.try_allocate_pages_at(new_phys, additional)
                        .unwrap_or_else(|| halt());
                    ptr::write_bytes(allocated as *mut u8, 0, additional * PAGE_SIZE as usize);
                    mappings[found_j].pages += additional;
                }
                existing_phys
            } else {
                let preferred: u64 = p_paddr & !PAGE_OFFSET_MASK;
                let physical_base_address: u64 = if preferred != 0 {
                    firmware.try_allocate_pages_at(preferred, pages)
                        .unwrap_or_else(|| firmware.allocate_pages(pages))
                } else {
                    firmware.allocate_pages(pages)
                };
                ptr::write_bytes(physical_base_address as *mut u8, 0, pages * PAGE_SIZE as usize);
                mappings[*mapping_count] = SegmentMapping { virtual_page_base, physical_base_address, pages };
                *mapping_count += 1;
                physical_base_address
            };
            ptr::copy_nonoverlapping(kernel_buffer.add(p_offset as usize), (dest_physical + page_offset) as *mut u8, p_filesz as usize);
        }
    }
}

unsafe fn apply_relocations(
    kernel_buffer: *mut u8,
    mappings: &[SegmentMapping],
    mapping_count: usize,
) {
    unsafe {
        let elf: &Elf64Header = &*(kernel_buffer as *const Elf64Header);
        let (shoff, shnum, shentsize): (u64, u16, u16) = (elf.e_shoff, elf.e_shnum, elf.e_shentsize);
        if shoff == 0 || shentsize == 0 {
            return;
        }
        for shdr_idx in 0..shnum as usize {
            let shdr_base: *const u8 = kernel_buffer.add(shoff as usize + shdr_idx * shentsize as usize);
            let sh_type: u32 = *(shdr_base.add(4) as *const u32);
            if sh_type != SHT_RELA {
                continue;
            }
            let sh_offset: u64 = *(shdr_base.add(24) as *const u64);
            let sh_size: u64 = *(shdr_base.add(32) as *const u64);
            let sh_entsize: u64 = *(shdr_base.add(56) as *const u64);
            if sh_entsize == 0 {
                continue;
            }
            let reloc_count: usize = sh_size as usize / sh_entsize as usize;
            for reloc_idx in 0..reloc_count {
                let reloc_entry: *const u64 = kernel_buffer.add(sh_offset as usize + reloc_idx * sh_entsize as usize) as *const u64;
                let r_offset: u64 = *reloc_entry;
                let r_info: u64 = *reloc_entry.add(1);
                let r_addend: i64 = *(reloc_entry.add(2) as *const i64);
                if r_info as u32 != ELF_R_RELATIVE {
                    continue;
                }
                let target_physical: u64 = virtual_to_physical(r_offset, &mappings[..mapping_count]);
                if target_physical != 0 {
                    *(target_physical as *mut u64) = r_addend as u64;
                }
            }
        }
    }
}

pub unsafe fn load_elf<F: FirmwareInterface>(
    firmware: &mut F,
    kernel_buffer: *mut u8,
    mappings: &mut [SegmentMapping; 16],
    mapping_count: &mut usize,
) -> u64 {
    unsafe {
        if *(kernel_buffer as *const u32) != ELF_MAGIC {
            halt();
        }
        let ei_class: u8 = *kernel_buffer.add(EI_CLASS);
        let entry: u64 = match ei_class {
            ELFCLASS64 => (*(kernel_buffer as *const Elf64Header)).e_entry,
            ELFCLASS32 => (*(kernel_buffer as *const Elf32Header)).e_entry as u64,
            _ => halt(),
        };

        load_segments(firmware, kernel_buffer, ei_class, mappings, mapping_count);

        if ei_class == ELFCLASS64 {
            apply_relocations(kernel_buffer, mappings, *mapping_count);
        }

        firmware.free_buffer(kernel_buffer);
        entry
    }
}
