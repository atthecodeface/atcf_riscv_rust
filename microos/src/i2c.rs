extern crate riscv_base;

const WAIT_SLEEP:u32 =1000;
//const WAIT_SLEEP:u32 =10;


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

#[allow(dead_code)]
pub fn vcu108_i2c_exec(gpio_base:u32, num_out:u32, num_in:u32, cont:bool, data_in:u32) -> (bool, u32) {
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

#[allow(dead_code)]
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
