fn uart_loopback() -> () {
    riscv_base::uart::config(70);
    loop {
        let rx = riscv_base::uart::rx();
        if rx<256 {
            let rx = if (rx>32) && (rx<127) {
                rx + 1
            } else {rx};
            riscv_base::uart::tx(rx);
        } else {
          if (rx&0x70000000)!=0 {
             riscv_base::uart::status();
          }
        }
        unsafe {riscv_base::sleep(100)};
    }
}

    // riscv_base::uart::config();

    // uart_loopback();
    // riscv_base::uart::tx_when_ready(65);
    // riscv_base::uart::tx_when_ready(10);
    // riscv_base::uart::tx_when_ready(13);
