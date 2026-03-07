# bootloader_from_scratch

A UEFI bootloader for x86-64 written from scratch in Rust, with no external dependencies.

## Prerequisites

- Rust nightly toolchain
- QEMU
- OVMF firmware

## Usage

**1. Build**

```bash
cargo build
```

The EFI binary is output to `target/x86_64-unknown-uefi/debug/bootloader_from_scratch.efi`.

**2. Create a FAT32 disk image**

The EFI System Partition (ESP) is a FAT32 partition where UEFI firmware looks for bootloaders.

```bash
dd if=/dev/zero of=disk.img bs=1M count=64
mkfs.fat -F 32 disk.img
```

**3. Populate the ESP**

```bash
mkdir -p mnt/EFI/BOOT
mount disk.img mnt
cp target/x86_64-unknown-uefi/debug/bootloader_from_scratch.efi mnt/EFI/BOOT/BOOTX64.EFI
cp <your_kernel> mnt/
echo "<your_kernel_filename>" > mnt/boot.cfg
umount mnt
```

`boot.cfg` must contain the filename of the kernel to load, for example:

```
ferrum
```

**4. Run with QEMU**

```bash
qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=<OVMF_CODE.fd> \
  -drive if=pflash,format=raw,file=<OVMF_VARS.fd> \
  -drive file=disk.img,format=raw
```
