
// address = 0
// data    = 8..11
// address commit = 16
// data commit    = 24..27

// 0x80450001 for 128.57MHz
// 53 for 100MHz
pub fn set_uart_brg(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_DPRINTF_UART, 1, data);
}

pub fn read_address() -> u32 {
    super::minimal::read_dev_apb(super::minimal::APB_DPRINTF, 16)
}

pub fn is_busy() -> bool {
    (read_address()>>31)!=0
}

pub fn wait() {
    while is_busy() {};
}

pub fn write1(address:u32, data:u32) {
    unsafe {
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF, 8, data);
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF,16,address);
    };
}

pub fn write4(address:u32, data:(u32, u32, u32, u32)) {
    unsafe {
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF, 8, data.0);
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF, 9, data.1);
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF,10, data.2);
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF,11, data.3);
        super::minimal::write_dev_apb(super::minimal::APB_DPRINTF,16,address);
    };
}
