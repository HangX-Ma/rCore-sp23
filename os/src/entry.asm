# os/src/entry.asm
    .section .text.entry
    .global _start
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .global boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .global boot_stack_top
boot_stack_top: