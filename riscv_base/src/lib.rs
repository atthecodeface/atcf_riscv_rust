#![no_std]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

extern crate libc;

extern {
    pub fn sleep(delay:u32) -> ();
}

