#![no_std]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

extern crate libc;

extern {
    pub fn asm_idle(count: u32) -> ();
    pub fn asm_cpp_cmd(cmd_address:u32, cpp_mem:*mut u32, cpp_address:u32) -> ();
    pub fn asm_cpp_cmd_direct(cmd_address:u32, cpp_address:u32) -> ();
    pub fn asm_read_any(signals:u32) -> u32;
    pub fn asm_read_all(signals:u32) -> u32;
    pub fn asm_wait_for_any(signals:u32) -> u32;
    pub fn asm_wait_for_all(wait:u32) -> u32;
    pub fn asm_wait_for_any_timeout(wait:u32) -> u32;
    pub fn asm_wait_for_all_timeout(wait:u32) -> u32;
    pub fn asm_read_write_timeout(timeout:u32) -> u32;
    pub fn test_pass() -> ();
    pub fn test_fail(failure:u32) -> ();
}

pub const CEP_XPB_BASE         : u32 = 0xc8000000;
pub const CEP_XPB_ISLAND_MASK  : u32 = 0x3f;
pub const CEP_XPB_ISLAND_SHIFT : u32 = 20;
pub const CEP_XPB_DEVICE_MASK  : u32 = 0xff;
pub const CEP_XPB_DEVICE_SHIFT : u32 = 12;
pub const CEP_XPB_REGISTER_MASK : u32 = 0xffc;

macro_rules! cep_xpb_idr {
 ($i:expr, $d:expr, $r:expr) => ( CEP_XPB_BASE |
    (($i&CEP_XPB_ISLAND_MASK)<<CEP_XPB_ISLAND_SHIFT)
        | (($d&CEP_XPB_DEVICE_MASK)<<CEP_XPB_DEVICE_SHIFT)
        | ($r&CEP_XPB_REGISTER_MASK) );
}

macro_rules! cep_apb_dr {
 ($d:expr, $r:expr) => (cep_xpb_idr! (2,0x10,($d<<8)|$r));
}

pub const SCR_W :u32= 60;
pub const SCR_H :u32= 40;

#[repr(C)]
struct ApbSram {
    address: u32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
    data_inc: u32,
}

pub const CEP_APB_TIMER:   *mut u32 = cep_apb_dr!(0,0) as *mut u32;
pub const CEP_APB_GPIO:    *mut u32 = cep_apb_dr!(1,0) as *mut u32;
pub const CEP_APB_DPRINTF: *mut u32 = cep_apb_dr!(2,0) as *mut u32;
pub const CEP_APB_FB_SRAM: *mut u32 = cep_apb_dr!(7,0) as *mut u32;

pub fn fb_clear_screen() {
    unsafe {
        let cep_fb_sram__address  : *mut u32 = CEP_APB_FB_SRAM.offset(0);
        let cep_fb_sram__data_inc : *mut u32 = CEP_APB_FB_SRAM.offset(4);
        core::ptr::write_volatile(cep_fb_sram__address, 0);
        for x in 1..(SCR_W * SCR_H) {
            core::ptr::write_volatile(cep_fb_sram__data_inc, 32);
        }
    }
}

pub fn fb_write_string(address: u32, data:&[u8]) {
    unsafe {
        let cep_fb_sram__address  : *mut u32 = CEP_APB_FB_SRAM.offset(0);
        let cep_fb_sram__data_inc : *mut u32 = CEP_APB_FB_SRAM.offset(4);
        core::ptr::write_volatile(cep_fb_sram__address, address);
        for x in data {
            core::ptr::write_volatile(cep_fb_sram__data_inc, *x as u32);
        }
    }
}

pub fn idle(count: u32) {
    unsafe {asm_idle(count);}
}

#[link_section = ".cpp"]
pub static mut my_cpp_data : [u32; 2] = [0, 0];

fn set_my_cpp_data(core:u32, ofs:isize, val:u32) {
    unsafe {
        let data_ptr : *mut u32 = my_cpp_data.as_mut_ptr().offset((core as isize)*256+ofs);
        core::ptr::write_volatile(data_ptr,val);
    }
}
fn get_my_cpp_data(core:u32, ofs:isize) -> u32 {
    let data = unsafe {
        let data_ptr : *const u32 = my_cpp_data.as_ptr().offset((core as isize)*256+ofs);
        core::ptr::read_volatile(data_ptr)
    };
    return data;
}

pub fn cls_clear_ticket (cls_address:u32) {
    unsafe {
        my_cpp_data[0] = 0;
        my_cpp_data[1] = 0;
        asm_cpp_cmd(0xe8201000,my_cpp_data.as_mut_ptr(),cls_address); // write of [0, 0]
        asm_wait_for_all(1<<6);
    };
}

pub fn cls_get_ticket (cls_address:u32, core: u32) {
    unsafe {
        set_my_cpp_data(core,0,0);
        set_my_cpp_data(core,1,1);
        //my_cpp_data[0] = 0;
        //my_cpp_data[1] = 1;
        asm_cpp_cmd(0xea801000,my_cpp_data.as_mut_ptr(),cls_address); // test_and_add32 of [0,1]test_and_add32 of [0,1]
        asm_wait_for_all(1<<6);
    };
    let mut ticket_being_served = get_my_cpp_data(core,0);
    let our_ticket              = get_my_cpp_data(core,1);
    while our_ticket != ticket_being_served {
        unsafe {
            idle(10<<15);
            asm_cpp_cmd(0xe8000000,my_cpp_data.as_mut_ptr(),cls_address); // read 32bit word
            asm_wait_for_all(1<<6);
            ticket_being_served = get_my_cpp_data(core,0);
        }
    }
}

pub fn cls_release_ticket (cls_address:u32) {
    unsafe { asm_cpp_cmd_direct(0xe8a01000,cls_address); } // add32_imm of 1 (my_cpp_data... is not required)
}

