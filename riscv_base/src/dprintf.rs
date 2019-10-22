
// address = 0
// data    = 8..11
// address commit = 16
// data commit    = 24..27

pub fn read_address() -> u32 {
    unsafe {
        let apb_dprintf_address: *const u32 = super::minimal::APB_DPRINTF.offset(16);
        core::ptr::read_volatile(apb_dprintf_address)
    }
}

pub fn is_busy() -> bool {
    (read_address()>>31)!=0
}

pub fn wait() {
    while is_busy() {};
}

pub fn write1(address:u32, data:u32) {
    unsafe {
        let apb_dprintf_address: *mut u32 = super::minimal::APB_DPRINTF.offset(16);
        let apb_dprintf_data0:   *mut u32 = super::minimal::APB_DPRINTF.offset(8);
        core::ptr::write_volatile(apb_dprintf_data0,data);
        core::ptr::write_volatile(apb_dprintf_address,address)
    };
}

pub fn write4(address:u32, data:(u32, u32, u32, u32)) {
    unsafe {
        let apb_dprintf_address: *mut u32 = super::minimal::APB_DPRINTF.offset(16);
        let apb_dprintf_data0:   *mut u32 = super::minimal::APB_DPRINTF.offset(8);
        let apb_dprintf_data1:   *mut u32 = super::minimal::APB_DPRINTF.offset(9);
        let apb_dprintf_data2:   *mut u32 = super::minimal::APB_DPRINTF.offset(10);
        let apb_dprintf_data3:   *mut u32 = super::minimal::APB_DPRINTF.offset(11);
        core::ptr::write_volatile(apb_dprintf_data0,data.0);
        core::ptr::write_volatile(apb_dprintf_data1,data.1);
        core::ptr::write_volatile(apb_dprintf_data2,data.2);
        core::ptr::write_volatile(apb_dprintf_data3,data.3);
        core::ptr::write_volatile(apb_dprintf_address,address)
    };
}
