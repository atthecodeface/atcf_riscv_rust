#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;
extern crate pxeboot;
use pxeboot::{loader, tftp};
mod axi_socket;
mod ethernet;

const OUR_IP  : u32 = 0x01010102;
const OUR_MAC : (u32, u16) = (0x44a84229, 0x88f0);

#[link_section=".start"]
#[export_name = "__main"]
pub extern "C" fn main() -> () {

    riscv_base::fb_sram::set_control(0x303fe);
    riscv_base::framebuffer::timing_configure( riscv_base::framebuffer::TIMINGS_2K );
    riscv_base::vcu108::configure_adv7511(); // not sim
    riscv_base::fb_sram::set_control(0x303c0);
    riscv_base::uart::config(70);
    let mut axi = riscv_base::axi4s::Axi::new(4095);
    let mut eth_rx_buf = [0u8; 64];
    let mut udp_pkt_buf = [0u8; 1024];
    let mut eth_rx = ethernet::EthernetRx::new(&mut eth_rx_buf);
    let mut tftp_socket = axi_socket::TftpSocketAxi::new((172,20,2,153), &mut udp_pkt_buf);
    axi.reset();
    riscv_base::dprintf::wait();
    unsafe { riscv_base::sleep(10000); }
    riscv_base::dprintf::write1(0,0x455448ff);
    riscv_base::ethernet::autonegotiate(33);
    unsafe { riscv_base::sleep(4<<24); }
    riscv_base::ethernet::autonegotiate(33);
    unsafe { riscv_base::sleep(4<<24); }
    riscv_base::ethernet::autonegotiate(33);
    riscv_base::fb_sram::set_control(0x303fe);
    riscv_base::analyzer::enable_source(6,3,1);
    loop {
    unsafe { riscv_base::sleep(1000); }
//    unsafe { riscv_base::sleep(4<<24); }
//     riscv_base::fb_sram::set_control(0x303c0);
//     riscv_base::ethernet::get_gasket_status();
//     riscv_base::fb_sram::set_control(0x303fe);
        if eth_rx.poll(&mut axi) {
// riscv_base::ethernet::get_rx_stats();
            if !eth_rx.check_dest_mac(OUR_MAC)  {
                riscv_base::dprintf::wait();
                riscv_base::dprintf::write2(0,(0x42616420,0x4d6163ff));
                eth_rx.discard(&mut axi);
            } else if eth_rx.is_arp_ipv4(OUR_IP) {
                riscv_base::dprintf::write2(0,(0x41525034,0x202020ff));
                eth_rx.discard(&mut axi);
            } else if eth_rx.is_simple_ipv4() { // hdr_csum ok && 
                riscv_base::dprintf::write2(0,(0x49505634,0x202020ff));
                tftp_socket.eth_poll(&mut axi, &mut eth_rx);
                eth_rx.discard(&mut axi);
            } else  {
                riscv_base::dprintf::write2(0,(0x44726f70,0x202020ff));
                eth_rx.discard(&mut axi);
            }
        }
    }
/*
        uart_console::poll(&mut base_console);
        if base_console.rx_buffer_ready() {
            execute_rx_buffer(&mut base_console);
            base_console.rx_buffer_reset();
        }
    }
    let mut okay = true;
    let mut buf : [u8;4096] = [0; 4096];
    let mut tx_buf : [u8;128] = [0; 128];
    let mut loader_buffer : [u8;1024] = [0;1024];
    let subloader = loader::DebugSubLoader {};
    let mut loader = loader::Loader::<loader::DebugSubLoader>::new(&mut loader_buffer);
    let mut tftp = tftp::Tftp::<tftp_posix::TftpSocketPosix>::new(&mut tftp_socket);
    loop {
        if tftp.poll() {
            match tftp.get_event(&mut buf) {
                tftp::TftpEvent::Connect        => loader.reset(),
                tftp::TftpEvent::Data(ofs,size) => {let _=loader.rx_data(&subloader, &buf[ofs..], size);},
                tftp::TftpEvent::Error          => tftp.reset(),
                tftp::TftpEvent::Idle           => (),
            }
        }
    }
*/
}
