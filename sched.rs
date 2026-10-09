//! Preemptive round-robin scheduler for kernel threads.
//! Context switching = returning a different trap frame from `trap_handler`.
use crate::hw;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

const STACK_SIZE: usize = 64 * 1024;
const FRAME_WORDS: usize = 34; // [0]=unused, [i]=x_i, [32]=mepc, [33]=mstatus
const FRAME_BYTES: usize = FRAME_WORDS * 8;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Ready,
    Running,
    Sleeping(u64), // wake-up tick
    Dead,
}

pub struct Task {
    pub id: usize,
    pub name: String,
    pub state: State,
    sp: usize,
    _stack: Vec<u8>,
}

struct Sched {
    tasks: Vec<Task>,
    current: usize, // index into tasks
    next_id: usize,
}

struct Global(UnsafeCell<Sched>);
unsafe impl Sync for Global {}
static SCHED: Global =
    Global(UnsafeCell::new(Sched { tasks: Vec::new(), current: 0, next_id: 1 }));

fn sched() -> &'static mut Sched {
    unsafe { &mut *SCHED.0.get() }
}

/// Register the currently running boot context as task 0 ("idle").
pub fn init() {
    hw::without_irq(|| {
        let s = sched();
        s.tasks.push(Task {
            id: 0,
            name: String::from("idle"),
            state: State::Running,
            sp: 0,
            _stack: Vec::new(),
        });
        s.current = 0;
    });
}

pub fn spawn(name: &str, entry: extern "C" fn(usize), arg: usize) -> usize {
    hw::without_irq(|| {
        let s = sched();
        let stack = vec![0u8; STACK_SIZE];
        let top = (stack.as_ptr() as usize + STACK_SIZE) & !15;
        let frame = (top - FRAME_BYTES) as *mut usize;
        unsafe {
            for i in 0..FRAME_WORDS {
                frame.add(i).write(0);
            }
            frame.add(1).write(task_exit as usize); // ra: returning from entry exits the task
            frame.add(10).write(arg); // a0
            frame.add(32).write(entry as usize); // mepc
            frame.add(33).write((3 << 11) | (1 << 7)); // mstatus: MPP=M, MPIE=1
        }
        let id = s.next_id;
        s.next_id += 1;
        s.tasks.push(Task {
            id,
            name: String::from(name),
            state: State::Ready,
            sp: frame as usize,
            _stack: stack,
        });
        id
    })
}

/// Called from the trap handler with the saved frame of the interrupted task.
/// Returns the frame to resume (possibly a different task's).
pub fn schedule(frame_sp: usize) -> usize {
    let s = sched();
    let cur_id = s.tasks[s.current].id;
    s.tasks[s.current].sp = frame_sp;
    if s.tasks[s.current].state == State::Running {
        s.tasks[s.current].state = State::Ready;
    }

    // wake sleepers
    let now = hw::ticks();
    for t in s.tasks.iter_mut() {
        if let State::Sleeping(w) = t.state {
            if now >= w {
                t.state = State::Ready;
            }
        }
    }

    // reap finished tasks (never the one whose stack we're still on)
    s.tasks.retain(|t| t.state != State::Dead || t.id == cur_id);
    let cur = s.tasks.iter().position(|t| t.id == cur_id).unwrap_or(0);

    // round-robin: next Ready task after `cur`
    let n = s.tasks.len();
    let mut next = cur;
    for off in 1..=n {
        let i = (cur + off) % n;
        if s.tasks[i].state == State::Ready {
            next = i;
            break;
        }
    }
    s.tasks[next].state = State::Running;
    s.current = next;
    s.tasks[next].sp
}

extern "C" fn task_exit() -> ! {
    hw::without_irq(|| {
        let s = sched();
        let c = s.current;
        s.tasks[c].state = State::Dead;
    });
    loop {
        hw::yield_now();
        hw::wfi();
    }
}

pub fn sleep_ms(ms: u64) {
    let wait = (ms * hw::TICK_HZ / 1000).max(1);
    let wake = hw::ticks() + wait;
    hw::without_irq(|| {
        let s = sched();
        let c = s.current;
        s.tasks[c].state = State::Sleeping(wake);
    });
    hw::yield_now();
}

pub fn current_id() -> usize {
    hw::without_irq(|| {
        let s = sched();
        s.tasks[s.current].id
    })
}

pub fn list() -> Vec<(usize, String, &'static str)> {
    hw::without_irq(|| {
        sched()
            .tasks
            .iter()
            .map(|t| {
                let st = match t.state {
                    State::Running => "running",
                    State::Ready => "ready",
                    State::Sleeping(_) => "sleeping",
                    State::Dead => "dead",
                };
                (t.id, t.name.clone(), st)
            })
            .collect()
    })
}
