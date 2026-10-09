#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
mod console;
mod fs;
mod heap;
mod hw;
mod sched;
mod shell;

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.S"));

extern "C" {
    fn trap_entry();
}

/// Called from trap_entry with a pointer to the saved register frame.
/// Returns the frame to resume (a different one = context switch).
#[no_mangle]
extern "C" fn trap_handler(frame: usize) -> usize {
    let mcause = hw::read_mcause();
    let top = usize::BITS - 1;
    let is_interrupt = (mcause >> top) == 1;
    let code = mcause & !(1usize << top);

    if is_interrupt {
        match code {
            7 => {
                // machine timer: advance time, preempt
                hw::tick();
                hw::schedule_next_tick();
                sched::schedule(frame)
            }
            3 => {
                // machine software interrupt: voluntary yield
                hw::clear_soft();
                sched::schedule(frame)
            }
            _ => panic!("unexpected interrupt {}", code),
        }
    } else {
        let mepc = unsafe { *(frame as *const usize).add(32) };
        panic!(
            "exception: cause={} mepc={:#x} mtval={:#x}",
            code,
            mepc,
            hw::read_mtval()
        );
    }
}

#[no_mangle]
extern "C" fn kmain() -> ! {
    heap::init();
    let (total, _) = heap::stats();
    println!("rvos 0.2 booting on RISC-V (machine mode)");
    println!("heap ready: {} MiB", total / (1024 * 1024));

    hw::set_trap_vector(trap_entry as usize);
    sched::init();
    fs::init();
    sched::spawn("shell", shell::run, 0);

    hw::schedule_next_tick();
    hw::enable_interrupts();

    // This boot context becomes task 0: the idle task.
    loop {
        hw::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hw::disable_irq();
    println!("\n*** KERNEL PANIC ***\n{}", info);
    loop {
        hw::wfi();
    }
}
