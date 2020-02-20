use super::tftp::{TftpSocket};
use super::ethernet::{EthernetRx, EthernetTx};
extern crate riscv_base;
use riscv_base::axi4s::Axi;

pub struct TftpSocketAxi <'a> {
    rx_pkt_buffer : &'a mut [u8;768],
    tx_pkt_buffer : &'a mut [u8;256],
    server_ipv4 : u32,
    server_port : u16,
    server_mac : (u32, u16),
    client_port : u16,
    rx_bytes_valid : usize,
    tx_bytes_valid : usize,
}

impl <'a> TftpSocketAxi <'a> {
    pub fn new(addr:(u8,u8,u8,u8), rx_pkt_buffer : &'a mut [u8;768], tx_pkt_buffer : &'a mut [u8;256]) -> TftpSocketAxi <'a> {
 
        let server_ipv4 = ((addr.0 as u32) << 24) |
                            ((addr.1 as u32) << 16) |        
                            ((addr.2 as u32) << 8) |        
                            ((addr.3 as u32) << 0);
        let server_port = 69;
        let server_mac = (0x44a84229, 0x88ef);
        let client_port = 12345;
        let rx_bytes_valid = 0;
        let tx_bytes_valid = 0;
        TftpSocketAxi {server_ipv4, server_port, server_mac, client_port, rx_pkt_buffer, tx_pkt_buffer, rx_bytes_valid, tx_bytes_valid}
    }
    pub fn eth_poll_rx(&mut self, axi:&mut Axi, eth_rx:&mut EthernetRx) -> bool {
        self.rx_bytes_valid = 0;
        if eth_rx.is_udp(self.client_port) {
            let src_ip   = eth_rx.ipv4_src_ip();
            if src_ip != self.server_ipv4 {
                false
            } else {
                self.server_mac   = eth_rx.eth_src_mac();
                self.server_port  = eth_rx.ipv4_src_port();
                self.rx_bytes_valid = eth_rx.copy_udp_payload(axi, self.rx_pkt_buffer);
                true
            }
        } else { false }
    }
    pub fn eth_poll_tx(&mut self, axi:&mut Axi, eth_tx:&mut EthernetTx, client_ipv4:u32)  {
        if self.tx_bytes_valid>0 {
            eth_tx.create_udp_ipv4_pkt_hdr(self.client_port, client_ipv4, self.server_mac, self.server_ipv4, self.server_port, self.tx_bytes_valid);
            eth_tx.transmit_with_data(axi, self.tx_pkt_buffer, self.tx_bytes_valid);
            self.tx_bytes_valid = 0;
        }
    }
}

impl <'a> TftpSocket for TftpSocketAxi <'a> {
    fn reset(&mut self) {
        self.server_port = 69;
    }
    fn has_rx_data(&self) -> bool {
        self.rx_bytes_valid > 0
    }
    fn rx_data(&mut self, data:&mut[u8]) -> usize {
        let size = data.len();
        let size = if size>self.rx_bytes_valid {self.rx_bytes_valid} else {size};
        for i in 0..size {
            data[i] = self.rx_pkt_buffer[i];
        }
        self.rx_bytes_valid = 0;
        size
    }
    fn can_tx_data(&self) -> bool {
        self.tx_bytes_valid == 0
    }
    fn tx_data(&mut self, data:&[u8]) -> usize {
        let size = data.len();
        let size = if size>self.tx_pkt_buffer.len() {self.tx_pkt_buffer.len()} else {size};
        for i in 0..size {
            self.tx_pkt_buffer[i] = data[i];
        }
        self.tx_bytes_valid = size;
            unsafe {
                let mut data_ptr: *mut u32  = self.tx_pkt_buffer.as_mut_ptr() as *mut u32;
                for _ in 0..16 {
                    let d = *data_ptr;
                    riscv_base::dprintf::write4(0,(0x87,d,0xffffffff,0));
                    data_ptr = data_ptr.offset(1);
                }
            }
        size
    }
}

