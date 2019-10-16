#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;

pub const APB_BASE         : u32 = 0x100000;

macro_rules! apb_dev {
 ($d:expr, $r:expr) => (APB_BASE | ($d<<16) | $r);
}

pub const APB_TIMER:            *mut u32 = apb_dev!(0,0) as *mut u32;
pub const APB_GPIO:             *mut u32 = apb_dev!(1,0) as *mut u32;
pub const APB_DPRINTF:          *mut u32 = apb_dev!(2,0) as *mut u32;
pub const APB_CSR:              *mut u32 = apb_dev!(3,0) as *mut u32;
pub const APB_RV_SRAM:          *mut u32 = apb_dev!(4,0) as *mut u32;
pub const APB_FB_SRAM:          *mut u32 = apb_dev!(7,0) as *mut u32;
pub const APB_UART:             *mut u32 = apb_dev!(9,0) as *mut u32;
pub const APB_DPRINTF_UART:     *mut u32 = apb_dev!(10,0) as *mut u32;
pub const APB_RISCV_DBG:        *mut u32 = apb_dev!(11,0) as *mut u32;

pub fn uart_config() {
    unsafe {
        let apb_uart_brg  : *mut u32 = APB_UART.offset(1);
        core::ptr::write_volatile(apb_uart_brg,162)
    };
}

pub fn uart_rx() -> u32 {
    return unsafe {
        let apb_uart_holding  : *const u32 = APB_UART.offset(3);
        core::ptr::read_volatile(apb_uart_holding)
    };
}

pub fn uart_tx(data:u32) {
    unsafe {
        let apb_uart_holding  : *mut u32 = APB_UART.offset(3);
        core::ptr::write_volatile(apb_uart_holding, data)
    };
}

#[link_section=".start"]
#[export_name = "__main"]
pub extern "C" fn main() -> () {
    uart_config();
    loop {
        let rx = uart_rx();
        if rx<256 {
            let rx = if (rx>32) && (rx<127) {
                rx + 1
            } else {rx};
            uart_tx(rx);
        }
        unsafe {riscv_base::sleep(100)};
    }
}
