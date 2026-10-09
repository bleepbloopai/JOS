OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY {
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 128M
}

SECTIONS {
    .text : {
        *(.text.init)
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(8) { *(.rodata .rodata.*) } > RAM

    .data : ALIGN(8) {
        *(.sdata .sdata.*)
        *(.data .data.*)
    } > RAM

    .bss : ALIGN(8) {
        _bss_start = .;
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        _bss_end = .;
    } > RAM

    /* boot stack (used by the idle/boot task) */
    .stack (NOLOAD) : ALIGN(16) {
        _stack_bottom = .;
        . += 64K;
        _stack_top = .;
    } > RAM

    /* everything after the stack, up to the end of RAM, is the kernel heap */
    _heap_start = ALIGN(16);
    _heap_end   = ORIGIN(RAM) + LENGTH(RAM);

    /DISCARD/ : { *(.eh_frame) *(.eh_frame_hdr) }
}
