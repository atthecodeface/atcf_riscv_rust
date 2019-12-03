pub fn autonegotiate(adv:u32) {
    super::rv_sram::set_control( (1<<28) | (0<<4) | 8 |1);
    super::rv_sram::set_control( (1<<28) | (0<<4) | 0 |1);
    super::rv_sram::set_control( (adv<<8) | (1<<4) | 8 |1);
    super::rv_sram::set_control( (adv<<8) | (1<<4) | 0 |1);
    super::rv_sram::set_control(   (1<<8) | (2<<4) | 8 |1);
    super::rv_sram::set_control(   (1<<8) | (2<<4) | 0 |1);
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

