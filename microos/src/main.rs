#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;

//const WAIT_SLEEP:u32 =1000;
const WAIT_SLEEP:u32 =10;

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

fn show_gpio_in() -> () {
   riscv_base::dprintf::wait();
   riscv_base::dprintf::write1(0,0x8100ffff|((riscv_base::gpio::get_inputs()&0xff)<<16));
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
        let r=(riscv_base::gpio::get_inputs()&1)==1;
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
   let nack = vcu108_i2c_output_bit(gpio_base, true); // output a 1 (others may pull it low)
   riscv_base::dprintf::wait();
   riscv_base::dprintf::write4(0,(0x41636b87, (if nack {0} else {1}), 0xffffffff,0));
   false
}

fn vcu108_i2c_exec(gpio_base:u32, num_out:u32, num_in:u32, cont:bool, data_in:u32) -> (bool, u32) {
    let mut okay = true;
    let mut data = data_in;
    vcu108_i2c_start(gpio_base);
    if num_out>0 {
        for _ in 0..num_out {
            if okay { okay = vcu108_i2c_output_byte(gpio_base, data);
                      data = data >> 8;
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
    (okay, 0)
}

fn vcu108_i2c_reset(gpio_base:u32) -> () {
    riscv_base::gpio::set_outputs(gpio_base | 0x20); // drive i2c_reset_mux_n low
    wait(10);
    riscv_base::gpio::set_outputs(gpio_base | 0x30); //  i2c_reset_mux_n high
    wait(10);
    riscv_base::gpio::set_outputs(gpio_base | 0); //  release i2c_reset_mux_n
    wait(10);
    for _ in 0..31 {
        vcu108_i2c_start(gpio_base);
    wait(10);
        vcu108_i2c_stop(gpio_base);
    wait(10);
        }
}

// write 0x0 to 0x75
// write 0x20 to 0x74
// write 16 bits of data regaddr/regdata to 0x39
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
    
    let gpio = riscv_base::gpio::get_outputs();
    let gpio_base = gpio & !0x3f;
    vcu108_i2c_reset(gpio_base);
    vcu108_i2c_exec(gpio_base, 1, 1, false, (0x75<<1)|1 );
    vcu108_i2c_reset(gpio_base);
    riscv_base::dprintf::wait();
    riscv_base::dprintf::write1(0,0x454e44ff);
    loop {};
}
