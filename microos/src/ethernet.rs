pub fn debug_send_pkt() {
    riscv_base::axi4s::write_tx_config(4095);
    riscv_base::axi4s::set_tx_ptr(1);
    riscv_base::axi4s::write_tx_data_inc(0); // user not used by GbE at present
    for i in 1..32 {
        riscv_base::axi4s::write_tx_data_inc(i); // 32 words of packet data
    }
    riscv_base::axi4s::write_tx_data_inc(0); // next packet start will be at 2+length in words
    riscv_base::axi4s::set_tx_ptr(0);
    riscv_base::axi4s::write_tx_data_inc(128); // number of bytes in packet
    riscv_base::axi4s::set_tx_ptr(34); // 2 + length in words of packet
}

pub fn autonegotiate(adv:u32) {
    riscv_base::rv_sram::set_control( (1<<28) | (0<<4) | 8 |1);
    riscv_base::rv_sram::set_control( (1<<28) | (0<<4) | 0 |1);
    riscv_base::rv_sram::set_control( (adv<<8) | (1<<4) | 8 |1);
    riscv_base::rv_sram::set_control( (adv<<8) | (1<<4) | 0 |1);
    riscv_base::rv_sram::set_control(   (1<<8) | (2<<4) | 8 |1);
    riscv_base::rv_sram::set_control(   (1<<8) | (2<<4) | 0 |1);
}

