pub fn autonegotiate(adv:u32) {
    riscv_base::rv_sram::set_control( (1<<28) | (0<<4) | 8 |1);
    riscv_base::rv_sram::set_control( (1<<28) | (0<<4) | 0 |1);
    riscv_base::rv_sram::set_control( (adv<<8) | (1<<4) | 8 |1);
    riscv_base::rv_sram::set_control( (adv<<8) | (1<<4) | 0 |1);
    riscv_base::rv_sram::set_control(   (1<<8) | (2<<4) | 8 |1);
    riscv_base::rv_sram::set_control(   (1<<8) | (2<<4) | 0 |1);
}

