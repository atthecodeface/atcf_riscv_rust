// timing
//    csr_address_display_size = 0,
//    csr_address_h_porch      = 1,
//    csr_address_v_porch      = 2,
//    csr_address_strobes      = 3,

// VGA timings require a 25MHz pixel clock
// HD 2k timings require a 150MHz pixel clock

pub const TIMINGS_2K : (u32, u32, u32, u32)  = (0x04380780, 0x005800c0, 0x00040029, 0x8005802c);
pub const TIMINGS_VGA : (u32, u32, u32, u32) = (0x01e00280, 0x00100090, 0x000a0023, 0x00020060);
pub fn timing_configure(timings:(u32, u32, u32, u32)) {
    unsafe {
        let r: *mut u32 = super::minimal::CSR_DEBUG_FB_TIM.offset(0);
        core::ptr::write_volatile(r, timings.0);
        let r: *mut u32 = super::minimal::CSR_DEBUG_FB_TIM.offset(1);
        core::ptr::write_volatile(r, timings.1);
        let r: *mut u32 = super::minimal::CSR_DEBUG_FB_TIM.offset(2);
        core::ptr::write_volatile(r, timings.2);
        let r: *mut u32 = super::minimal::CSR_DEBUG_FB_TIM.offset(3);
        core::ptr::write_volatile(r, timings.3);
    };
}
