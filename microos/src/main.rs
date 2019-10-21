#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;


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
   loop {};
}
fn vcu108_i2c_reset(gpio_base:u32) -> () {
    riscv_base::gpio::set_outputs(gpio_base | 0x20); // drive i2c_reset_mux_n low
    wait(10);
    riscv_base::gpio::set_outputs(gpio_base | 0x30); //  i2c_reset_mux_n high
    wait(10);
    riscv_base::gpio::set_outputs(gpio_base | 0); //  release i2c_reset_mux_n
    wait(10);
}

fn vcu108_i2c_start(gpio_base:u32) -> () {
    riscv_base::gpio::set_outputs(gpio_base | 0x8); //  Start (SDA low, SCL stays high)
    wait(2);
    riscv_base::gpio::set_outputs(gpio_base | 0xa); //  SDA low SCL low
    wait(2);
}

fn vcu108_i2c_stop(gpio_base:u32) -> () {
    riscv_base::gpio::set_outputs(gpio_base | 0xa); //  SDA low SCL low
    wait(2);
    riscv_base::gpio::set_outputs(gpio_base | 0x8); //  SCL high, SDA low
    wait(2);
    riscv_base::gpio::set_outputs(gpio_base | 0);   //  SDA high, SCL high
    wait(2);
}

fn vcu108_i2c_output_bit(gpio_base:u32, data:bool) -> () {
    riscv_base::gpio::set_outputs(gpio_base | (0x2 | (if data {8} else {0}))); //  Keep SCL low, SDA to bit value
    wait(2);
    riscv_base::gpio::set_outputs(gpio_base | (0x0 | (if data {8} else {0}))); //  Let SCL float high, SDA hold
    wait(2);
    riscv_base::gpio::set_outputs(gpio_base | (0x2 | (if data {8} else {0}))); //  Pull SCL low, SDA hold
    wait(2);
}

fn vcu108_i2c_read_bit(gpio_base:u32, data:bool) -> bool {
    riscv_base::gpio::set_outputs(gpio_base | 0x2 ); //  Keep SCL low, SDA float
    wait(2);
    riscv_base::gpio::set_outputs(gpio_base | 0x0 ); //  Let SCL float high
    wait(1);
    //let r=riscv_base::gpio_bit;
    wait(1);
    riscv_base::gpio::set_outputs(gpio_base | 0x2 ); //  Pull SCL low, SDA hold
    wait(2);
    true
}

fn vcu108_i2c_output_byte(gpio_base:u32, data:u32) -> bool {
   for i in 0..7 {
     vcu108_i2c_output_bit(gpio_base, ((data>>i)&1)==1 );
   }
   true
}


// ; // write
    // 7 bits + RnW
//    vcu108_i2c_output_bit(gpio_base, 0);
    // Ac
//    let ack = vcu108_i2c_read_bit(gpio_base);
//    if ack

// write 0x0 to 0x75
// write 0x20 to 0x74
// write 16 bits of data regaddr/regdata to 0x39
#[link_section=".start"]
#[export_name = "__main"]
pub extern "C" fn main() -> () {

    uart_loopback();
    let mut gpio = riscv_base::gpio::get_outputs();
    let gpio_base = gpio & !0x3f;

    vcu108_i2c_reset(gpio_base);
    vcu108_i2c_start(gpio_base);
    if ( (vcu108_i2c_output_byte(gpio_base, (0x75<<1)|0)) &&
         (vcu108_i2c_output_byte(gpio_base, 0))
         ) {
      vcu108_i2c_stop(gpio_base);
    }
}
