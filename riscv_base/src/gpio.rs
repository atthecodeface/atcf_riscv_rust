
//    apb_address_gpio_output_reg   = 0,
//    apb_address_gpio_input_status = 1,
//    apb_address_gpio_input_reg_0  = 2,
//    apb_address_gpio_input_reg_1  = 3,

pub fn set_otputs() {
    unsafe {
        let apb_uart_brg  : *mut u32 = super::minimal::APB_UART.offset(1);
        core::ptr::write_volatile(apb_uart_brg,53) // 53 for 100MHz to 115.2kHz
    };
}

pub fn uart_status() -> u32 {
    return unsafe {
        let apb_uart_status  : *const u32 = super::minimal::APB_UART.offset(0);
        core::ptr::read_volatile(apb_uart_status)
    };
}

pub fn get_outputs() -> u32 {
    return unsafe {
        let gpio_output  : *mut u32 = super::minimal::APB_GPIO.offset(0);
        core::ptr::read_volatile(gpio_output)
    };
}

// pairs of bits for each I/O: values are 0x (undriven), 10 (low), 11 (high)
pub fn set_outputs(data:u32) {
    unsafe {
        let gpio_output  : *mut u32 = super::minimal::APB_GPIO.offset(0);
        core::ptr::write_volatile(gpio_output, data)
    };
}

pub fn drive_pin(pin:u32, value:bool) {
    unsafe {
        let shift = pin*2;
        let gpio_output  : *mut u32 = super::minimal::APB_GPIO.offset(0);
        let r = core::ptr::read_volatile(gpio_output) & !(3<<shift);
        core::ptr::write_volatile(gpio_output, r | ((if value {3} else {2})<<shift));
    };
}

