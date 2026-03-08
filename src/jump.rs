use core::arch::global_asm;
use core::mem;

use crate::gdt::constants::GDT32;
use crate::gdt::gdtr::Gdtr;

pub unsafe fn jump_32bit(entry: u32, eax: u32, ebx: u32) -> ! {
    unsafe {
        let gdtr: Gdtr = Gdtr {
            limit: (mem::size_of::<[u64; 3]>() - 1) as u16,
            base: GDT32.as_ptr() as u64,
        };
        _jump32_trampoline(entry, eax, ebx, &gdtr as *const Gdtr as u64);
    }
}

pub unsafe fn jump_64bit(entry: u64) -> ! {
    unsafe {
        core::arch::asm!(
            "jmp {entry}",
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}


unsafe extern "efiapi" {
    fn _jump32_trampoline(entry: u32, eax: u32, ebx: u32, gdtr_ptr: u64) -> !;
}

global_asm!(r#"
.code64
.global _jump32_trampoline
_jump32_trampoline:
    cli
    lgdt [r9]

    mov ebp, ecx
    mov esi, edx
    mov edi, r8d

    lea rax, [rip + .Ljump32_entry]
    push 8
    push rax
    .byte 0x48, 0xcb

.code32
.Ljump32_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov eax, cr0
    and eax, 0x7FFFFFFF
    mov cr0, eax

    mov ecx, 0xC0000080
    rdmsr
    and eax, 0xFFFFFEFF
    wrmsr

    mov eax, esi
    mov ebx, edi
    jmp ebp
"#);
