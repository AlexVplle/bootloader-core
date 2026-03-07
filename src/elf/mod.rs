pub mod elf32;
pub mod elf64;

pub const EI_CLASS: usize = 4;
pub const ELFCLASS32: u8 = 1;
pub const ELFCLASS64: u8 = 2;
