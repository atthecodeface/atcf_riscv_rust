
pub fn config(brg_cfg:u32) {
    unsafe {
        let apb_uart_brg  : *mut u32 = super::minimal::APB_UART.offset(1);
        core::ptr::write_volatile(apb_uart_brg,brg_cfg) // 53 for 100MHz to 115.2kHz
    };
}

pub fn status() -> u32 {
    return unsafe {
        let apb_uart_status  : *const u32 = super::minimal::APB_UART.offset(0);
        core::ptr::read_volatile(apb_uart_status)
    };
}

pub fn rx() -> u32 {
    return unsafe {
        let apb_uart_holding  : *const u32 = super::minimal::APB_UART.offset(3);
        core::ptr::read_volatile(apb_uart_holding)
    };
}

pub fn tx(data:u32) {
    unsafe {
        let apb_uart_holding  : *mut u32 = super::minimal::APB_UART.offset(3);
        core::ptr::write_volatile(apb_uart_holding, data)
    };
}

pub fn tx_when_ready(data:u32) {
    loop {
        if (status()&0x100)==0 { break; }
        }
    tx(data)
}

