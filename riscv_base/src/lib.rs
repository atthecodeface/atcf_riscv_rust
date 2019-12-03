#![no_std]

pub mod minimal;
pub mod uart;
pub mod gpio;
pub mod i2c_master;
pub mod fb_sram;
pub mod rv_sram;
pub mod dprintf;
pub mod framebuffer;
pub mod axi4s;
pub mod ethernet;

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

extern crate libc;

extern {
    pub fn sleep(delay:u32) -> ();
}

