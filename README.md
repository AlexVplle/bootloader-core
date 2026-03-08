# bootloader_from_scratch

A UEFI bootloader for x86-64 written from scratch in Rust, with no external dependencies.

## Supported boot protocols

- **Linux** (bzImage, EFI handover protocol)
- **Limine** (protocol v1, framebuffer + memory map)
- **Multiboot2**
- **Multiboot1**

The protocol is auto-detected from the kernel binary.

## Prerequisites

- Rust nightly toolchain with `x86_64-unknown-uefi` target
- QEMU with OVMF firmware

## Usage

**1. Build**

```bash
cargo build
```

The EFI binary is output to `target/x86_64-unknown-uefi/debug/bootloader_from_scratch.efi`.

**2. Create a FAT32 disk image**

```bash
dd if=/dev/zero of=disk.img bs=1M count=64
mkfs.fat -F 32 disk.img
```

**3. Populate the disk**

```bash
mkdir -p mnt/EFI/BOOT
mount disk.img mnt
cp target/x86_64-unknown-uefi/debug/bootloader_from_scratch.efi mnt/EFI/BOOT/BOOTX64.EFI
cp <your_kernel> mnt/kernel
umount mnt
```

The bootloader looks for a file named `kernel` at the root of the FAT32 partition.

**4. Run with QEMU**

```bash
qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=<OVMF_CODE.fd> \
  -drive if=pflash,format=raw,file=<OVMF_VARS.fd> \
  -drive file=disk.img,format=raw \
  -m 512
```

Use `-m 1024` or more when booting Linux (needed for kernel decompression).

## References

- [UEFI Specification](https://uefi.org/specifications)
- [Multiboot2 Specification](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html)
- [Linux boot protocol](https://www.kernel.org/doc/html/latest/arch/x86/boot.html)
- [Limine boot protocol](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)
