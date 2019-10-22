
// address = 0
// data = 1
// control = 2
// data_inc = 4...7
// windowed=128..255

pub fn set_control(data:u32) {
    unsafe {
        let apb_sram_ctrl : *mut u32 = super::minimal::APB_FB_SRAM.offset(2);
        core::ptr::write_volatile(apb_sram_ctrl,data)
    };
}

pub fn get_control() -> u32 {
    return unsafe {
        let apb_sram_ctrl : *const u32 = super::minimal::APB_FB_SRAM.offset(2);
        core::ptr::read_volatile(apb_sram_ctrl)
    };
}
