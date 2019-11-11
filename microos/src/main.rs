#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;

const WAIT_SLEEP:u32 =1000;
//const WAIT_SLEEP:u32 =10;

fn uart_loopback() -> () {
    riscv_base::uart::config();
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

fn wait(v:u32) -> () {
   for _ in 0..v {
       unsafe {riscv_base::sleep(WAIT_SLEEP)};
   }
}

fn vcu108_i2c_drive_and_wait(gpio:u32, delay:u32) -> () {
    riscv_base::gpio::set_outputs(gpio);
    wait(delay);
}

fn vcu108_i2c_start(gpio_base:u32) -> () {
    vcu108_i2c_drive_and_wait(gpio_base | 0x0, 2);   //  SDA high, SCL high
    vcu108_i2c_drive_and_wait(gpio_base | 0x8, 2);   //  SDA low, SCL high
    vcu108_i2c_drive_and_wait(gpio_base | 0xa, 2);   //  SDA low, SCL low
}

fn vcu108_i2c_stop(gpio_base:u32) -> () {
    vcu108_i2c_drive_and_wait(gpio_base | 0xa, 2);   //  SDA low, SCL low
    vcu108_i2c_drive_and_wait(gpio_base | 0x8, 2);   //  SDA low, SCL high
    vcu108_i2c_drive_and_wait(gpio_base | 0x0, 2);   //  SDA high, SCL high
}

fn vcu108_i2c_cont(gpio_base:u32) -> () {
    vcu108_i2c_drive_and_wait(gpio_base | 0xa, 2);   //  SDA low, SCL low
    vcu108_i2c_drive_and_wait(gpio_base | 0x2, 2);   //  SDA high, SCL low
    vcu108_i2c_drive_and_wait(gpio_base | 0x0, 2);   //  SDA high, SCL high
}

fn vcu108_i2c_output_bit(gpio_base:u32, data:bool) -> bool {
    if data {
        vcu108_i2c_drive_and_wait(gpio_base | 0x2, 2);   //  SDA high, SCL low
        vcu108_i2c_drive_and_wait(gpio_base | 0x0, 2);   //  SDA high, SCL high
        let r=(riscv_base::gpio::get_inputs()&2)!=0;
        vcu108_i2c_drive_and_wait(gpio_base | 0x2, 2);   //  SDA high, SCL low
        r
    } else {
        vcu108_i2c_drive_and_wait(gpio_base | 0xa, 2);   //  SDA low, SCL low
        vcu108_i2c_drive_and_wait(gpio_base | 0x8, 2);   //  SDA low, SCL high
        vcu108_i2c_drive_and_wait(gpio_base | 0xa, 2);   //  SDA low, SCL low
        false
    }
}

fn vcu108_i2c_output_byte(gpio_base:u32, data:u32) -> bool {
   for i in 0..8 {
     vcu108_i2c_output_bit(gpio_base, ((data>>(7-i))&1)==1 );
   }
   // output a 1 during NACK cycle (others may pull it low)
   !vcu108_i2c_output_bit(gpio_base, true)
}

fn vcu108_i2c_input_byte(gpio_base:u32, data_in:u32, last:bool) -> (bool, u32) {
   let mut data=data_in;
   for i in 0..8 {
     data = (data << 1) | (if vcu108_i2c_output_bit(gpio_base, true) {1} else {0});
   }
   vcu108_i2c_output_bit(gpio_base, last); // nack the last one
   (true, data)
}

fn vcu108_i2c_exec(gpio_base:u32, num_out:u32, num_in:u32, cont:bool, data_in:u32) -> (bool, u32) {
    let mut okay = true;
    let mut data = data_in;
    vcu108_i2c_start(gpio_base);
    if num_out>0 {
        for _ in 0..num_out {
            if okay {
                 okay = vcu108_i2c_output_byte(gpio_base, data);
                 data = data >> 8;
            }
        }
    }
    if okay && (num_in>0) {
        data = 0;
        for i in 0..num_in {
            if okay {
                let r = vcu108_i2c_input_byte(gpio_base,
                     data,
                     i==(num_in-1) );
                okay = r.0;
                data = r.1;
            }
        }
    }
    if okay {
        if cont {
            vcu108_i2c_cont(gpio_base);
        } else {
            vcu108_i2c_stop(gpio_base);
        }    
    } else {
        vcu108_i2c_stop(gpio_base);
    }
   riscv_base::dprintf::wait();
   riscv_base::dprintf::write4(0, (0x81003a87 | (if okay {0x10000} else {0}),
                                                    data,0xffffffff,0)
                                                        );
    (okay, data)
}

fn vcu108_i2c_reset(gpio_base:u32) -> () {
    riscv_base::gpio::set_outputs(gpio_base | 0x20); // drive i2c_reset_mux_n low
    wait(10);
    riscv_base::gpio::set_outputs(gpio_base | 0x30); //  i2c_reset_mux_n high
    wait(10);
    riscv_base::gpio::set_outputs(gpio_base | 0); //  release i2c_reset_mux_n
    wait(10);
    vcu108_i2c_start(gpio_base);
    for _ in 0..31 {
        vcu108_i2c_output_bit(gpio_base, true); // clock with no ack and just high data
    }
    vcu108_i2c_stop(gpio_base);
}

// write 0x0 to 0x75
// write 0x20 to 0x74
// write 16 bits of data regaddr/regdata to 0x39

// ADV7511 in XCVU108 connects data bits [16;8] and hence is 16-bit (INPUT ID=1, style=1)
// This requires Cb/Y, Cr/Y in successive cycles
// I2C address is 0x72
// Out of reset (when powers up)
// I2c register 0x41[1;6] = power down (must only be cleared when HPD is high)
// I2C register 0x98[8;0] = 0x03
// I2C register 0x9a[7;1] = 0x70
// I2C register 0x9c[7;0] = 0x30
// I2C register 0x9d[2;0] = 0x01
// I2C register 0xa2[8;0] = 0xa4
// I2C register 0xa3[8;0] = 0xa4
// I2C register 0xe0[8;0] = 0xd0
// I2c register 0xaf[1;1] = output is HDMI (not DVI)
// Input style
// I2C register 0xf9[8;0] = 0
// I2C register 0x15[4;0] = 1 (input id)  (other bits 0)
// I2C register 0x16[2;4] = 2b11 (8 bit per channel)
// I2C register 0x16[2;2] = 2b10 (style 1)  (other bits 0)
// I2C register 0x48[2;3] = 01 (right justified) (other bits 0)
// 1080p-60 is 1920x1080
// pixel clock 148.5MHz = 900MHz/6 almost
// line time is 14.8us
// frame time is 16666us
// horizontal front porch/sync/back porch/active = 88/44/148/1920 +ve sync
// vertical   front porch/sync/back porch/active = 4/5/36/1080 +ve sync

const adv7511_init : [u32; 15] = [ 0xc0d6, 0x1041, 0x0398, 0xe09a, 0x309c, 0x619d, 0xa4a2, 0xa4a3, 0xd0e0, 0x00f9, 0x0115, 0x3416, 0x0848, 0x02af, 0x0217 ];

fn configure_adv7511_old() {
    let gpio = riscv_base::gpio::get_outputs();
    let gpio_base = gpio & !0x3f;
    //vcu108_i2c_reset(gpio_base);
    // vcu108_i2c_exec(gpio_base, 1, 1, false, (0x75<<1)|1 );
    // vcu108_i2c_exec(gpio_base, 1, 1, false, (0x74<<1)|1 );
    // Disable 4-port I2C expander
    vcu108_i2c_exec(gpio_base, 2, 0, false, (0x0000)|(0x75<<1)|0 );
    // Enable 8-port I2C expander to talk to ADV7511 only
    vcu108_i2c_exec(gpio_base, 2, 0, false, (0x2000)|(0x74<<1)|0 );
    // Write to ADV7511 (note can set d6[2;6] to 11 to have 'HPD is always high')
    // Note 98-ae, cd-f8 are not reset with HPD
    for w in &adv7511_init {
        vcu108_i2c_exec(gpio_base, 3, 0, false, (w<<8)|(0x39u32<<1)|0u32 );
    }
    vcu108_i2c_exec(gpio_base, 2, 0, true, (0x00<<8)|(0x39<<1)|0 );
    vcu108_i2c_exec(gpio_base, 1, 1, false, (0x39<<1)|1 );
    vcu108_i2c_exec(gpio_base, 2, 0, true, (0x3c<<8)|(0x39<<1)|0 );
    vcu108_i2c_exec(gpio_base, 1, 1, false, (0x39<<1)|1 );
    vcu108_i2c_exec(gpio_base, 2, 0, true, (0x3d<<8)|(0x39<<1)|0 );
    vcu108_i2c_exec(gpio_base, 1, 1, false, (0x39<<1)|1 );
    vcu108_i2c_exec(gpio_base, 2, 0, true, (0x3e<<8)|(0x39<<1)|0 );
    vcu108_i2c_exec(gpio_base, 1, 1, false, (0x39<<1)|1 );
    //vcu108_i2c_reset(gpio_base);
    //riscv_base::fb_sram::set_control((1<<6)); // enable vsync
}

fn configure_adv7511() {
    // for 100MHz clock the divider can be 10, and period 10 (i.e. 10MHz I2C pin sampling, 1us period for master transitions)
    // period_delay of 10 gives 50kHz I2C interface - give a hold of 6 and setup of 4 to split period for when SDA changes
    riscv_base::i2c_master::write_i2c_config(0x046a0a0a);
    // Disable 4-port I2C expander
    riscv_base::i2c_master::exec(2, 0, false, (0x0000)|(0x75<<1)|0 );
    // Enable 8-port I2C expander to talk to ADV7511 only
    riscv_base::i2c_master::exec(2, 0, false, (0x2000)|(0x74<<1)|0 );
    // Write to ADV7511 (note can set d6[2;6] to 11 to have 'HPD is always high')
    // Note 98-ae, cd-f8 are not reset with HPD
    for w in &adv7511_init {
        riscv_base::i2c_master::exec(3, 0, false, (w<<8)|(0x39u32<<1)|0u32 );
    }
    riscv_base::i2c_master::exec(2, 0, true, (0x00<<8)|(0x39<<1)|0 );
    riscv_base::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
    riscv_base::i2c_master::exec(2, 0, true, (0x3c<<8)|(0x39<<1)|0 );
    riscv_base::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
    riscv_base::i2c_master::exec(2, 0, true, (0x3d<<8)|(0x39<<1)|0 );
    riscv_base::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
    riscv_base::i2c_master::exec(2, 0, true, (0x3e<<8)|(0x39<<1)|0 );
    riscv_base::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
}

fn debug_send_pkt() {
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

#[link_section=".start"]
#[export_name = "__main"]
pub extern "C" fn main() -> () {

    // riscv_base::fb_sram::set_control((1<<11)|(1<<6)); // kill VGA and APB dprintfs
    // riscv_base::uart::config();

    // uart_loopback();
    // riscv_base::uart::tx_when_ready(65);
    // riscv_base::uart::tx_when_ready(10);
    // riscv_base::uart::tx_when_ready(13);
    // riscv_base::dprintf::write4(0,(0x414243ff,0,0,0));
    // riscv_base::dprintf::wait();
    // riscv_base::fb_sram::set_control((1<<11)|(0<<6)); // kill VGA
    //    riscv_base::fb_sram::set_control(1<<11);
    //    riscv_base::gpio::get_inputs();
    //    riscv_base::fb_sram::set_control((1<<11)|(1<<6));
    riscv_base::framebuffer::timing_configure( riscv_base::framebuffer::TIMINGS_2K );

    configure_adv7511();
    //riscv_base::dprintf::set_uart_brg(0x80450001);
    riscv_base::dprintf::set_uart_brg(69);
    riscv_base::axi4s::write_rx_config(4095);
    riscv_base::dprintf::wait();
    riscv_base::dprintf::write1(0,0x454e44ff);
    let mut lg = 0;
    let mut n = 0;
    riscv_base::fb_sram::set_control((1<<12)|(1<<11)|(1<<6));
    loop {
        unsafe {riscv_base::sleep(100000)};
        let g = riscv_base::gpio::get_inputs();
        if g!=lg {
           riscv_base::dprintf::wait();
           riscv_base::dprintf::write4(30,(0x87,g,0xffffffff,0xffffffff));
           lg = g;
           if (g&0x10)!=0 {
               debug_send_pkt();
           }
           if (g&0x100)!=0 {
    riscv_base::fb_sram::set_control((1<<12)|(1<<11));
               let d=riscv_base::axi4s::read_rx_data();
               riscv_base::dprintf::wait();
               riscv_base::dprintf::write4(20,(0x87,d,0xffffffff,0xffffffff));
    riscv_base::fb_sram::set_control((1<<12)|(1<<11)|(1<<6));
           }
        }
    };
}
