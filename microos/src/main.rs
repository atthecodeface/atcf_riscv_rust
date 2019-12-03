#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;
mod i2c;
mod adv7511;
mod uart;
mod uart_console;

fn parse_int(line : &[u8], okay : bool) -> ( u32, bool, &[u8]) {
    let l = line.len();
    if !okay || (l==0) {
        (0, false, line)
    } else if line[0]==32 {
        parse_int(&line[1..], okay)
    } else {
        let mut d:u32=0;
        let mut i=0;
        while i<l {
            let c=line[i] as u32;
            if (c>=48) && (c<=57) { d=(d<<4)|(c-48); } 
            else if ((c&0xdf)>=65) && ((c&0xdf)<=70) { d=(d<<4)|((c&0xdf)-55); }
            else {break};
            i += 1;
        }
        if i==0 {(0, false, line)} else {(d, true, &line[i..])}
    }
}

fn execute_rx_buffer(console : &mut uart_console::Console) {
    let line = console.get_rx_buffer();
    if line.len()>0 {
        let c = line[0];
        if c==64+23 {
            let (address,okay,rest) = parse_int(&line[1..], true);
            let (data,okay,rest)    = parse_int(&rest, okay);
            if okay {
                unsafe {
                    let r: *mut u32 = address as *mut u32;
                    core::ptr::write_volatile(r.offset(0), data);
                }
                console.tx_buffer_push(64+23);
            } else {
                console.tx_buffer_push(96+23);
            }
        } else if c==64+18 {
            let (address,okay,rest) = parse_int(&line[1..], true);
            if okay {
                console.tx_buffer_push(64+18);
                let data =     unsafe {
                    let r: *const u32 = address as *const u32;
                    core::ptr::read_volatile(r.offset(0))
                };

                for i in 0..8 {
                    let c = (data>>(4*(7-i))) & 0xf;
                    let c = {if c<10 {c+48} else {c+55}};
                    console.tx_buffer_push(c as u8);
                }
            } else {
                console.tx_buffer_push(96+18);
            }
        }
    }
}

#[link_section=".start"]
#[export_name = "__main"]
pub extern "C" fn main() -> () {

    // riscv_base::fb_sram::set_control((1<<11)|(1<<6)); // kill VGA and APB dprintfs
    // riscv_base::dprintf::write4(0,(0x414243ff,0,0,0));
    // riscv_base::dprintf::wait();
    // riscv_base::fb_sram::set_control((1<<11)|(0<<6)); // kill VGA
    //    riscv_base::fb_sram::set_control(1<<11);
    //    riscv_base::gpio::get_inputs();
    //    riscv_base::fb_sram::set_control((1<<11)|(1<<6));
    riscv_base::framebuffer::timing_configure( riscv_base::framebuffer::TIMINGS_2K );

    adv7511::configure_adv7511(); // not sim
    riscv_base::ethernet::autonegotiate(33);
    let mut axi = riscv_base::axi4s::Axi::new(4095);
    axi.reset();
    riscv_base::dprintf::wait();
    riscv_base::dprintf::write1(0,0x454e44ff);
    riscv_base::fb_sram::set_control((1<<12)|(1<<11));
    let d=riscv_base::axi4s::read_rx_data();
    //riscv_base::dprintf::wait();
    riscv_base::dprintf::write4(20,(0x87,d,0xffffffff,0xffffffff));
    riscv_base::fb_sram::set_control((1<<12)|(1<<11)|(1<<6));

    /*
    let mut lg = 0;
    let mut n = 0;
    //riscv_base::fb_sram::set_control((1<<12)|(1<<11)|(1<<6));
    loop {
    unsafe {riscv_base::sleep(100000)}; // not sim
    let g = riscv_base::gpio::get_inputs();
    if g!=lg {
    riscv_base::dprintf::wait(); //  not sim
    riscv_base::dprintf::write4(30,(0x87,g,0xffffffff,0xffffffff));
    lg = g;
    if (g&0x80)!=0 {
    ethernet::autonegotiate(0xf0ff);
}
    if (g&0x10)!=0 {
    ethernet::debug_send_pkt();
}
    if (g&0x100)!=0 {
    let d=riscv_base::axi4s::read_rx_data();
    riscv_base::dprintf::wait(); //  not sim
    riscv_base::dprintf::write4(20,(0x87,d,0xffffffff,0xffffffff));
}
}
};
     */
    riscv_base::uart::config(70);
    let mut base_console = uart_console::Console{
        tx_ptrs:uart_console::Buffer{size:64,count:0,read:0,write:0,ready:false,hack:0},
        rx_ptrs:uart_console::Buffer{size:64,count:0,read:0,write:0,ready:false,hack:0},
        tx_buffer:[0;64],
        rx_buffer:[0;64],
    };
    loop {
        uart_console::poll(&mut base_console);
        if base_console.rx_buffer_ready() {
            execute_rx_buffer(&mut base_console);
            /*                while !base_console.rx_buffer_empty() {
            let d = base_console.rx_buffer_pop();
            base_console.tx_buffer_push(d);
        }
             */
            base_console.rx_buffer_reset();
        }
    }
}
