# bootloader-core

Firmware-agnostic bootloader core library written in Rust, with no external dependencies.

Handles ELF loading, page table construction, and boot protocol detection/boot. Designed to be used by firmware-specific entry points via the `FirmwareInterface` trait.

## Supported boot protocols

- **Limine** (protocol v1, framebuffer + memory map)
- **Multiboot2**
- **Multiboot1**

## Usage

Add as a dependency in your firmware-specific crate:

```toml
[dependencies]
bootloader-core = { git = "https://github.com/AlexVplle/bootloader_from_scratch.git" }
```

Implement `FirmwareInterface` for your firmware:

```rust
impl FirmwareInterface for MyFirmware {
    unsafe fn allocate_pages(&mut self, count: usize) -> u64 { ... }
    unsafe fn try_allocate_pages_at(&mut self, address: u64, count: usize) -> Option<u64> { ... }
    unsafe fn free_buffer(&mut self, ptr: *mut u8) { ... }
}
```

Then call `load_elf`, `build_page_tables`, and the appropriate protocol boot function.

## References

- [Multiboot2 Specification](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html)
- [Limine boot protocol](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)
