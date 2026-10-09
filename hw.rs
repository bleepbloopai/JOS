//! Hardware access for the QEMU "virt" RISC-V machine (machine mode only).
use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

pub const CLINT_MSIP: usize = 0x0200_0000;
pub const CLINT_MTIMECMP: usize = 0x0200_4000; // hart 0
pub const CLINT_MTIME: usize = 0x0200_BFF8;
const TEST_FINISHER: usize = 0x10_0000;

const MTIME_HZ: u64 = 10_000_000;
pub const TICK_HZ: u64 = 100; // scheduler tick = 10 ms

const MSTATUS_MIE: usize = 1 << 3;
const MIE_MSIE: usize = 1 << 3; // software interrupt (used for yield)
const MIE_MTIE: usize = 1 << 7; // timer interrupt

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn ticks() -> u64 {
    TICKS.load(Ordering::SeqCst)
}
pub fn tick() {
    TICKS.fetch_add(1, Ordering::SeqCst);
}

/// Run `f` with machine interrupts disabled, restoring the previous state after.
pub fn without_irq<R>(f: impl FnOnce() -> R) -> R {
    let prev: usize;
    unsafe { asm!("csrrc {0}, mstatus, {1}", out(reg) prev, in(reg) MSTATUS_MIE, options(nostack)) };
    let r = f();
    if prev & MSTATUS_MIE != 0 {
        unsafe { asm!("csrs mstatus, {0}", in(reg) MSTATUS_MIE, options(nostack)) };
    }
    r
}

pub fn disable_irq() {
    unsafe { asm!("csrc mstatus, {0}", in(reg) MSTATUS_MIE, options(nostack)) };
}

pub fn set_trap_vector(addr: usize) {
    unsafe { asm!("csrw mtvec, {0}", in(reg) addr, options(nostack)) };
}

/// Enable the timer + software interrupts, then global interrupts.
pub fn enable_interrupts() {
    unsafe {
        asm!("csrs mie, {0}", in(reg) MIE_MSIE | MIE_MTIE, options(nostack));
        asm!("csrs mstatus, {0}", in(reg) MSTATUS_MIE, options(nostack));
    }
}

pub fn read_mcause() -> usize {
    let v: usize;
    unsafe { asm!("csrr {0}, mcause", out(reg) v, options(nomem, nostack)) };
    v
}
pub fn read_mtval() -> usize {
    let v: usize;
    unsafe { asm!("csrr {0}, mtval", out(reg) v, options(nomem, nostack)) };
    v
}

pub fn wfi() {
    unsafe { asm!("wfi", options(nostack)) };
}

pub fn schedule_next_tick() {
    unsafe {
        let now = (CLINT_MTIME as *const u64).read_volatile();
        (CLINT_MTIMECMP as *mut u64).write_volatile(now + MTIME_HZ / TICK_HZ);
    }
}

/// Trigger a software interrupt -> trap -> scheduler runs immediately.
pub fn yield_now() {
    unsafe { (CLINT_MSIP as *mut u32).write_volatile(1) };
}
pub fn clear_soft() {
    unsafe { (CLINT_MSIP as *mut u32).write_volatile(0) };
}

pub fn poweroff() -> ! {
    unsafe { (TEST_FINISHER as *mut u32).write_volatile(0x5555) };
    loop {
        wfi();
    }
}
pub fn reboot() -> ! {
    unsafe { (TEST_FINISHER as *mut u32).write_volatile(0x7777) };
    loop {
        wfi();
    }
}
