//! A flat in-memory filesystem (name -> bytes). Contents vanish on reboot.
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

struct Fs(UnsafeCell<BTreeMap<String, Vec<u8>>>);
unsafe impl Sync for Fs {}
static FS: Fs = Fs(UnsafeCell::new(BTreeMap::new()));

pub fn fs() -> &'static mut BTreeMap<String, Vec<u8>> {
    unsafe { &mut *FS.0.get() }
}

pub fn init() {
    fs().insert(
        String::from("readme.txt"),
        b"Welcome to rvos! Try: ls, cat readme.txt, write hi.txt hello, spawn 3, ps, mem".to_vec(),
    );
}
