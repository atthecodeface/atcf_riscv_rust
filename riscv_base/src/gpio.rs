
//    apb_address_gpio_output_reg   = 0,
//    apb_address_gpio_input_status = 1,
//    apb_address_gpio_input_reg_0  = 2,
//    apb_address_gpio_input_reg_1  = 3,

pub fn get_inputs() -> u32 {
    return unsafe {
        let gpio_input_status : *const u32 = super::minimal::APB_GPIO.offset(1);
        core::ptr::read_volatile(gpio_input_status)
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

