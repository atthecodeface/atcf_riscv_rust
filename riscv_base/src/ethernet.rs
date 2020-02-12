fn set_gasket_control(data:u32) {
    unsafe {
        let r : *mut u32 = super::minimal::APB_GBE.offset(1);
        core::ptr::write_volatile(r,data);
        unsafe {super::sleep(10);}
    };
}

pub fn get_gasket_status() -> u32 {
    return unsafe {
        let r : *const u32 = super::minimal::APB_GBE.offset(0);
        core::ptr::read_volatile(r)
    };
}

pub fn get_rx_stats() -> (u32, u32, u32) {
    let okay_pkts = unsafe {
        let r : *const u32 = super::minimal::APB_GBE.offset(12);
        core::ptr::read_volatile(r)
    };
    let okay_bytes = unsafe {
        let r : *const u32 = super::minimal::APB_GBE.offset(13);
        core::ptr::read_volatile(r)
    };
    let errored = unsafe {
        let r : *const u32 = super::minimal::APB_GBE.offset(14);
        core::ptr::read_volatile(r)
    };
    (okay_pkts, okay_bytes, errored)
}

pub fn autonegotiate(adv:u32) {
    set_gasket_control( ((150*10*1000)<<4) | 2 ); // 10ms timer
    set_gasket_control( (adv<<4) | 1 );           // advertised values
    set_gasket_control( (5<<4) | 0 );             // enable interface, with autonegotiation on
}

pub fn no_autonegotiate() {
    set_gasket_control( (7<<4) | 0 );             // enable interface, with autonegotiation off
}

pub fn reset(sram_size:u32) {
    super::axi4s::set_tx_ptr(0);
    super::axi4s::write_tx_data_inc(0); // user not used by GbE at present
    super::axi4s::write_tx_config(sram_size);
    super::axi4s::write_rx_config(sram_size);
}

pub fn debug_send_pkt() {
    super::axi4s::write_tx_config(4095);
    super::axi4s::set_tx_ptr(1);
    super::axi4s::write_tx_data_inc(0); // user not used by GbE at present
    for i in 1..32 {
        super::axi4s::write_tx_data_inc(i); // 32 words of packet data
    }
    super::axi4s::write_tx_data_inc(0); // next packet start will be at 2+length in words
    super::axi4s::set_tx_ptr(0);
    super::axi4s::write_tx_data_inc(128); // number of bytes in packet
    super::axi4s::set_tx_ptr(34); // 2 + length in words of packet
}

