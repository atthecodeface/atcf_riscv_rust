use super::tftp::{TftpSocket};
use super::ethernet::{EthernetRx};
extern crate riscv_base;
use riscv_base::axi4s::Axi;

pub struct TftpSocketAxi <'a> {
    pkt_buffer : &'a mut [u8;1024],
    server_ipv4 : u32,
    client_ipv4 : u32,
    server_port : u16,
    client_port : u16,
    bytes_valid : usize,
}

impl <'a> TftpSocketAxi <'a> {
    pub fn new(addr:(u8,u8,u8,u8), pkt_buffer : &mut [u8;1024]) -> TftpSocketAxi {
        let server_ipv4 = 0;
        let client_ipv4 = 0;
        let server_port = 69;
        let client_port = 12345;
        let bytes_valid = 0;
        TftpSocketAxi {server_ipv4, client_ipv4, server_port, client_port, pkt_buffer, bytes_valid}
    }
    pub fn eth_poll(&mut self, axi:&mut Axi, eth_rx:&mut EthernetRx) -> bool {
        // if udp && dest_ip == client_ipv4 && src_ip==server_ipv4 && dest_port==client_port && udp csum okay && long enough
        eth_rx.rx_packet(axi,self.pkt_buffer);
        true
    }
}

impl <'a> TftpSocket for TftpSocketAxi <'a> {
    fn reset(&mut self) {
        self.server_port = 69;
    }
    fn has_rx_data(&self) -> bool {
        self.bytes_valid > 0
    }
    fn rx_data(&mut self, data:&mut[u8]) -> usize {
        0 //self.bytes_valid-
    }
    fn can_tx_data(&self) -> bool {
        true
    }
    fn tx_data(&mut self, data:&[u8]) -> usize {
        //self.axi.tx_start_packet();
        //axi.tx_write_u32(rx_d);
        //axi.tx_send_packet();
        //size
        0
    }
}

