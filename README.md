# rvos 0.2 — a small multitasking RISC-V kernel in Rust

## Setup (once)
    rustup target add riscv64imac-unknown-none-elf
    # QEMU: `sudo apt install qemu-system-misc`  or  `brew install qemu`

## Run
    cargo run
Quit with the `poweroff` command, or Ctrl-A then X.

## Features
- Boot, UART console (output + polled input), panic handler
- Heap allocator (free list + coalescing) -> `alloc` works (Vec, String, BTreeMap, Box)
- Preemptive round-robin scheduler (10 ms tick), sleep, task exit/reaping
- Interactive shell with an in-memory filesystem

## Layout
- src/boot.S   entry point + full-context trap entry (context switch happens here)
- src/hw.rs    CLINT timer, interrupt control, power off/reboot
- src/console.rs  UART, print!/println!
- src/heap.rs  global allocator
- src/sched.rs scheduler / tasks
- src/fs.rs    RAM filesystem
- src/shell.rs shell commands
- src/main.rs  trap handler + kernel entry

## Try
    help
    spawn 5       # background task prints while you keep typing
    ps
    write a.txt hello world
    cat a.txt
    mem
