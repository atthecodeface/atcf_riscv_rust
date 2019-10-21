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
