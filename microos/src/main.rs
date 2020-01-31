#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;
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

static mut POLL_FLAGS:u32 = 0;
static mut POLL_RESULT:u32 = 0;
fn write_poll(d:u32) {
   unsafe {POLL_FLAGS = d;}
}

fn clear_poll(d:u32) {
   unsafe {POLL_FLAGS = POLL_FLAGS & !d;}
}

fn read_poll() -> u32 {
   unsafe {POLL_FLAGS}
}

fn write_poll_result(d:u32) {
   unsafe {POLL_RESULT = d;}
}

fn read_poll_result() -> u32 {
   unsafe {POLL_RESULT}
}

fn execute_rx_buffer(console : &mut uart_console::Console) {
    let line = console.get_rx_buffer();
    if line.len()>0 {
        let c = line[0];
        if c==64+23 { // W = APB write
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
        } else if c==64+18 { // R = APB read
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
        } else if c==64+16 { // P = poll rx
            let (value,okay,rest) = parse_int(&line[1..], true);
            if okay {
                let data = read_poll_result();
                write_poll(value);
                riscv_base::dprintf::write4(40,(0x20202020,0x20202020,0x20202020,0x202020ff));
                console.tx_buffer_push(64+16);
                for i in 0..8 {
                    let c = (data>>(4*(7-i))) & 0xf;
                    let c = {if c<10 {c+48} else {c+55}};
                    console.tx_buffer_push(c as u8);
                }
            } else {
                console.tx_buffer_push(96+16);
            }
        }
    }
}

// This code works
fn axi_poll_reflect (axi : &mut riscv_base::axi4s::Axi) {
    if (axi.rx_poll()) {
        let do_tx = (read_poll() & 2) != 0;
        let size = axi.rx_start_packet();
        unsafe { riscv_base::sleep(100); }
        riscv_base::dprintf::wait();
        riscv_base::dprintf::write4(40,(0x504B543a,0x2087,size,0xff));
        if (do_tx) {axi.tx_start_packet();}
        for i in 0..size {
            let rx_d = axi.rx_read_u32_raw();
            if do_tx && (i>0) {axi.tx_write_u32_raw(rx_d);}
            unsafe { riscv_base::sleep(100); }
            riscv_base::dprintf::wait();
            riscv_base::dprintf::write4(12*40+40*i,(0x87,rx_d,0xff,0));
        }
        axi.rx_end_packet();
        axi.tx_send_packet_raw((size-1)<<2);
        if (do_tx) {axi.tx_start_packet();}
        clear_poll(3);
    }
}

#[link_section=".start"]
#[export_name = "__main"]
pub extern "C" fn main() -> () {

    riscv_base::framebuffer::timing_configure( riscv_base::framebuffer::TIMINGS_2K );
    riscv_base::fb_sram::set_control(0xc2);
    riscv_base::vcu108::configure_adv7511(); // not sim
    riscv_base::dprintf::wait();
    riscv_base::dprintf::write1(0,0x454e44ff);

    riscv_base::uart::config(70);
    let mut base_console = uart_console::Console{
        tx_ptrs:uart_console::Buffer{size:64,count:0,read:0,write:0,ready:false,hack:0},
        rx_ptrs:uart_console::Buffer{size:64,count:0,read:0,write:0,ready:false,hack:0},
        tx_buffer:[0;64],
        rx_buffer:[0;64],
    };
    let mut axi = riscv_base::axi4s::Axi::new(4095);
    axi.reset();
    riscv_base::dprintf::wait();
    unsafe { riscv_base::sleep(10000); }
    riscv_base::dprintf::write1(0,0x455048ff);
    riscv_base::fb_sram::set_control(0xf8);
    riscv_base::ethernet::autonegotiate(33);

    riscv_base::fb_sram::set_control(0xfe);
    loop {
        if (read_poll() & 1)!=0 {
           if (axi.rx_poll()) {
               let do_tx = (read_poll() & 2) != 0;
               let size = axi.rx_start_packet();
               unsafe { riscv_base::sleep(100); }
               riscv_base::dprintf::wait();
               riscv_base::dprintf::write4(40,(0x504B543a,0x2087,size,0xff));
               if (do_tx) {axi.tx_start_packet();}
               for i in 0..size {
                   let rx_d = axi.rx_read_u32_raw();
                   if do_tx && (i>0) {axi.tx_write_u32(rx_d);} // not raw, so it updates tx_size by 1 each time
                   unsafe { riscv_base::sleep(100); }
                   riscv_base::dprintf::wait();
                   riscv_base::dprintf::write4(12*40+40*i,(0x87,rx_d,0xff,0));
               }
               axi.rx_end_packet();
               // axi.tx_send_packet_raw((size-1)<<2);
               axi.tx_send_packet(); // not raw, so it uses tx_size<<2 as the byte size
               if (do_tx) {axi.tx_start_packet();}
               clear_poll(3);
           }
        }
        uart_console::poll(&mut base_console);
        if base_console.rx_buffer_ready() {
    riscv_base::fb_sram::set_control(0xf8);
            execute_rx_buffer(&mut base_console);
    riscv_base::fb_sram::set_control(0xfe);
            base_console.rx_buffer_reset();
        }
    }
}
