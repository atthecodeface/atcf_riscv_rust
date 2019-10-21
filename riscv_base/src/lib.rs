#![no_std]

pub mod minimal;
pub mod uart;
pub mod gpio;

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

extern crate libc;

extern {
    pub fn sleep(delay:u32) -> ();
}

