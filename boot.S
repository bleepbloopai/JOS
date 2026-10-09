# Entry point: QEMU starts every hart here in machine mode.
.section .text.init
.global _start
_start:
    csrr    t0, mhartid
    bnez    t0, park            # only hart 0 continues

    la      sp, _stack_top

    la      t0, _bss_start      # zero .bss
    la      t1, _bss_end
1:  bgeu    t0, t1, 2f
    sd      zero, 0(t0)
    addi    t0, t0, 8
    j       1b

2:  call    kmain

park:
    wfi
    j       park

# Trap entry. Saves the FULL context (x1,x3..x31, mepc, mstatus) on the
# current stack as a 272-byte frame, calls trap_handler(frame_sp), and
# resumes from whatever frame pointer it returns. Returning a different
# frame is how context switching works.
.section .text
.align 4
.global trap_entry
trap_entry:
    addi    sp, sp, -272
    sd      x1, 8(sp)
    sd      x3, 24(sp)
    sd      x4, 32(sp)
    sd      x5, 40(sp)
    sd      x6, 48(sp)
    sd      x7, 56(sp)
    sd      x8, 64(sp)
    sd      x9, 72(sp)
    sd      x10, 80(sp)
    sd      x11, 88(sp)
    sd      x12, 96(sp)
    sd      x13, 104(sp)
    sd      x14, 112(sp)
    sd      x15, 120(sp)
    sd      x16, 128(sp)
    sd      x17, 136(sp)
    sd      x18, 144(sp)
    sd      x19, 152(sp)
    sd      x20, 160(sp)
    sd      x21, 168(sp)
    sd      x22, 176(sp)
    sd      x23, 184(sp)
    sd      x24, 192(sp)
    sd      x25, 200(sp)
    sd      x26, 208(sp)
    sd      x27, 216(sp)
    sd      x28, 224(sp)
    sd      x29, 232(sp)
    sd      x30, 240(sp)
    sd      x31, 248(sp)
    csrr    t0, mepc
    sd      t0, 256(sp)
    csrr    t0, mstatus
    sd      t0, 264(sp)

    mv      a0, sp
    call    trap_handler        # returns the frame to resume in a0
    mv      sp, a0

    ld      t0, 256(sp)
    csrw    mepc, t0
    ld      t0, 264(sp)
    csrw    mstatus, t0
    ld      x1, 8(sp)
    ld      x3, 24(sp)
    ld      x4, 32(sp)
    ld      x5, 40(sp)
    ld      x6, 48(sp)
    ld      x7, 56(sp)
    ld      x8, 64(sp)
    ld      x9, 72(sp)
    ld      x10, 80(sp)
    ld      x11, 88(sp)
    ld      x12, 96(sp)
    ld      x13, 104(sp)
    ld      x14, 112(sp)
    ld      x15, 120(sp)
    ld      x16, 128(sp)
    ld      x17, 136(sp)
    ld      x18, 144(sp)
    ld      x19, 152(sp)
    ld      x20, 160(sp)
    ld      x21, 168(sp)
    ld      x22, 176(sp)
    ld      x23, 184(sp)
    ld      x24, 192(sp)
    ld      x25, 200(sp)
    ld      x26, 208(sp)
    ld      x27, 216(sp)
    ld      x28, 224(sp)
    ld      x29, 232(sp)
    ld      x30, 240(sp)
    ld      x31, 248(sp)
    addi    sp, sp, 272
    mret
