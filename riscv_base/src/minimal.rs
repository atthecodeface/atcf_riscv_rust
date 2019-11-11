pub const APB_BASE         : u32 = 0x100000;

#[macro_export]
macro_rules! apb_dev {
 ($d:expr) => (APB_BASE | ($d<<16));
}

macro_rules! csr_dev {
 ($d:expr) => (APB_BASE | (3<<16) | ($d<<12));
}

pub const APB_TIMER:            *mut u32 = apb_dev!(0) as *mut u32;
pub const APB_GPIO:             *mut u32 = apb_dev!(1) as *mut u32;
pub const APB_CSR:              *mut u32 = apb_dev!(3) as *mut u32;
pub const APB_RV_SRAM:          *mut u32 = apb_dev!(4) as *mut u32;
pub const APB_FB_SRAM:          *mut u32 = apb_dev!(7) as *mut u32;
pub const APB_UART:             *mut u32 = apb_dev!(9) as *mut u32;
pub const APB_RISCV_DBG:        *mut u32 = apb_dev!(11) as *mut u32;
pub const APB_I2C_MASTER:       *mut u32 = apb_dev!(12) as *mut u32;
pub const APB_DPRINTF:      u32=2;
pub const APB_DPRINTF_UART: u32=10;
pub const APB_AXI4S:        u32=13;

pub const CSR_DEBUG_FB_DISP:     *mut u32 = csr_dev!(2) as *mut u32;
pub const CSR_DEBUG_FB_TIM:      *mut u32 = csr_dev!(3) as *mut u32;
pub const CSR_USER_FB_DISP:      *mut u32 = csr_dev!(4) as *mut u32;
pub const CSR_USER_FB_TIM:       *mut u32 = csr_dev!(5) as *mut u32;

pub fn write_dev_apb(dev:u32, offset:isize, data:u32) {
    unsafe {
        let r: *mut u32 = apb_dev!(dev) as *mut u32;
        core::ptr::write_volatile(r.offset(offset), data);
    }
}

pub fn read_dev_apb(dev:u32, offset:isize) -> u32 {
    unsafe {
        let r: *mut u32 = apb_dev!(dev) as *mut u32;
        core::ptr::read_volatile(r.offset(offset))
    }
}
