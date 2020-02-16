use super::tftp::{TftpSocket};
use super::ethernet::{EthernetRx};
extern crate riscv_base;
use riscv_base::axi4s::Axi;

pub struct TftpSocketAxi <'a> {
    pkt_buffer : &'a mut [u8;1024],
    server_ipv4 : u32,
    server_port : u16,
    client_port : u16,
    bytes_valid : usize,
}

impl <'a> TftpSocketAxi <'a> {
    pub fn new(addr:(u8,u8,u8,u8), pkt_buffer : &mut [u8;1024]) -> TftpSocketAxi {
        let server_ipv4 = 0;
        let server_port = 69;
        let client_port = 12345;
        let bytes_valid = 0;
        TftpSocketAxi {server_ipv4, server_port, client_port, pkt_buffer, bytes_valid}
    }
    pub fn eth_poll(&mut self, axi:&mut Axi, eth_rx:&mut EthernetRx) -> bool {
        if eth_rx.is_udp(self.client_port) {
            let src_ip   = eth_rx.ipv4_src_ip();
            let src_port = eth_rx.ipv4_src_port();
            if src_ip != self.server_ipv4 {
                false
            } else {
                self.bytes_valid = eth_rx.copy_udp_payload(axi,self.pkt_buffer);
                true
            }
        } else { false }
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
        let size = data.len();
        let size = if size>self.bytes_valid {self.bytes_valid} else {size};
        for i in 0..size {
            data[i] = self.pkt_buffer[i];
        }
        size
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

