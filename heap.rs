//! Kernel heap: sorted free-list allocator with coalescing.
//! All blocks are multiples of 16 bytes and 16-byte aligned (align > 16 unsupported).
use crate::hw;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{addr_of, null_mut};

extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
}

#[repr(C)]
struct Node {
    size: usize,
    next: *mut Node,
}
const MIN_BLOCK: usize = 16;

struct Heap {
    head: *mut Node,
    total: usize,
    used: usize,
}

fn align_up(x: usize, a: usize) -> usize {
    (x + a - 1) & !(a - 1)
}

impl Heap {
    const fn new() -> Self {
        Heap { head: null_mut(), total: 0, used: 0 }
    }

    unsafe fn init(&mut self, start: usize, end: usize) {
        let start = align_up(start, 16);
        let end = end & !15;
        self.total = end - start;
        self.insert(start, end - start);
    }

    fn block_size(layout: &Layout) -> usize {
        align_up(layout.size().max(1), 16).max(MIN_BLOCK)
    }

    unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        if layout.align() > 16 {
            return null_mut();
        }
        let size = Self::block_size(&layout);
        let mut prev: *mut Node = null_mut();
        let mut cur = self.head;
        while !cur.is_null() {
            if (*cur).size >= size {
                let rem = (*cur).size - size;
                let next = if rem >= MIN_BLOCK {
                    let n = (cur as usize + size) as *mut Node;
                    (*n).size = rem;
                    (*n).next = (*cur).next;
                    n
                } else {
                    (*cur).next
                };
                if prev.is_null() {
                    self.head = next;
                } else {
                    (*prev).next = next;
                }
                self.used += size;
                return cur as *mut u8;
            }
            prev = cur;
            cur = (*cur).next;
        }
        null_mut()
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        let size = Self::block_size(&layout);
        self.used -= size;
        self.insert(ptr as usize, size);
    }

    /// Insert a free block in address order and merge with neighbours.
    unsafe fn insert(&mut self, addr: usize, size: usize) {
        let mut prev: *mut Node = null_mut();
        let mut cur = self.head;
        while !cur.is_null() && (cur as usize) < addr {
            prev = cur;
            cur = (*cur).next;
        }
        let node = addr as *mut Node;
        (*node).size = size;
        (*node).next = cur;
        if prev.is_null() {
            self.head = node;
        } else {
            (*prev).next = node;
        }
        if !cur.is_null() && addr + size == cur as usize {
            (*node).size += (*cur).size;
            (*node).next = (*cur).next;
        }
        if !prev.is_null() && prev as usize + (*prev).size == addr {
            (*prev).size += (*node).size;
            (*prev).next = (*node).next;
        }
    }
}

pub struct KernelAlloc(UnsafeCell<Heap>);
unsafe impl Sync for KernelAlloc {}

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        hw::without_irq(|| (*self.0.get()).allocate(layout))
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        hw::without_irq(|| (*self.0.get()).deallocate(ptr, layout))
    }
}

#[global_allocator]
static ALLOCATOR: KernelAlloc = KernelAlloc(UnsafeCell::new(Heap::new()));

pub fn init() {
    unsafe {
        let start = addr_of!(_heap_start) as usize;
        let end = addr_of!(_heap_end) as usize;
        (*ALLOCATOR.0.get()).init(start, end);
    }
}

/// (total bytes, used bytes)
pub fn stats() -> (usize, usize) {
    hw::without_irq(|| unsafe {
        let h = &*ALLOCATOR.0.get();
        (h.total, h.used)
    })
}
