#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate riscv_base;
extern crate pxeboot;
use pxeboot::loader::Loader;
use pxeboot::tftp::{Tftp, TftpEvent};
use axi_socket::{TftpSocketAxi};
mod axi_socket;
mod ethernet;
mod loader;
use loader::{SubLoader};

const SERVER_IP : (u8,u8,u8,u8) = (1,1,1,1);//(172,20,2,153);
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
    let mut eth_tx_buf = [0u8; 64];
    let mut udp_rx_pkt_buf = [0u8; 768];
    let mut udp_tx_pkt_buf = [0u8; 256];
    let mut payload_buf    = [0u8; 1024];
    let mut loader_buffer  = [0u8; 1024];
    let mut eth_rx = ethernet::EthernetRx::new(&mut eth_rx_buf);
    let mut eth_tx = ethernet::EthernetTx::new(&mut eth_tx_buf);
    let mut tftp_socket = TftpSocketAxi::new(SERVER_IP, &mut udp_rx_pkt_buf, &mut udp_tx_pkt_buf);
    let mut tftp = Tftp::new();
    let mut loader = Loader::<SubLoader>::new(&mut loader_buffer);
    let subloader = SubLoader::new();
    eth_tx.set_src(OUR_MAC);
    axi.reset();
    riscv_base::dprintf::wait();
    unsafe { riscv_base::sleep(10000); }
    riscv_base::dprintf::write1(0,0x455448ff);
    riscv_base::ethernet::autonegotiate(33);
    riscv_base::fb_sram::set_control(0x303fe);
    riscv_base::analyzer::enable_source(6,3,1);
    let mut loop_count : usize = 0;
    loop {
        unsafe { riscv_base::sleep(1000); }
        loop_count = loop_count+1;
        if loop_count>0x40000 {
            riscv_base::fb_sram::set_control(0x303c0);
            let gasket_status = riscv_base::ethernet::get_gasket_status();
            if (gasket_status&0xffff)!=0xd801 {
                riscv_base::ethernet::disable();
                riscv_base::ethernet::autonegotiate(33);
            }
            riscv_base::ethernet::get_rx_stats();
            riscv_base::ethernet::get_tx_stats();
            riscv_base::fb_sram::set_control(0x303fe);
            loop_count = 0;
        }
        if eth_rx.poll(&mut axi) {
            if !eth_rx.check_dest_mac(OUR_MAC)  {
                riscv_base::dprintf::wait();
                riscv_base::dprintf::write2(0,(0x42616420,0x4d6163ff));
            } else if eth_rx.is_arp_ipv4(OUR_IP) {
                riscv_base::dprintf::write2(0,(0x41525034,0x202020ff));
                eth_tx.create_arp_reply(&eth_rx, OUR_MAC);
                eth_tx.transmit(&mut axi);
            } else if eth_rx.is_simple_ipv4(OUR_IP) { // hdr_csum ok && 
                riscv_base::dprintf::write2(0,(0x49505634,0x202020ff));
                tftp_socket.eth_poll_rx(&mut axi, &mut eth_rx);
            } else  {
                riscv_base::dprintf::write2(0,(0x44726f70,0x202020ff));
            }
            eth_rx.discard(&mut axi);
        }
        tftp_socket.eth_poll_tx(&mut axi, &mut eth_tx, OUR_IP);
        if tftp.poll(&mut tftp_socket) {
            let tftp_event = tftp.get_event(&mut tftp_socket, &mut payload_buf);
            match tftp_event {
                TftpEvent::Connect        => {
                    riscv_base::dprintf::write2(0,(0x54667470,0x20434fff));
                    loader.reset();
                },
                TftpEvent::Data(ofs,size) => {
                    riscv_base::dprintf::write2(0,(0x54667470,0x204441ff));
                    let _=loader.rx_data(&subloader, &payload_buf[ofs..], size);
                },
                TftpEvent::Error          => {
                    riscv_base::dprintf::write2(0,(0x54667470,0x204552ff));
                    tftp.reset();
                },
                TftpEvent::Idle           => {
                    //riscv_base::dprintf::write2(0,(0x54667470,0x204964ff));
                },
            }
        }
    }
}
