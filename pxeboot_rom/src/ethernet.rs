extern crate riscv_base;
use riscv_base::axi4s::Axi;

pub fn autonegotiate(adv:u32) {
    riscv_base::rv_sram::set_control( (1<<28) | (0<<4) | 8 |1);
    riscv_base::rv_sram::set_control( (1<<28) | (0<<4) | 0 |1);
    riscv_base::rv_sram::set_control( (adv<<8) | (1<<4) | 8 |1);
    riscv_base::rv_sram::set_control( (adv<<8) | (1<<4) | 0 |1);
    riscv_base::rv_sram::set_control(   (1<<8) | (2<<4) | 8 |1);
    riscv_base::rv_sram::set_control(   (1<<8) | (2<<4) | 0 |1);
}

pub struct EthernetRx <'a> {
    data : &'a mut [u8;64],
    bytes_valid : usize,
    packet_bytes : usize,
}

/*
	0x0000:  4500 0118 1efb 4000 4011 bc19 ac14 0299
	0x0010:  ac14 03ff 05fe 05fe 0104 5fd6 5443 4632
	0x0020:  0200 0000 4944 3d54 4350 3a31 3732 2e32
	0x0030:  302e 322e 3135 333a 3331 3231 004e 616d
	0x0040:  653d 5869 6c69 6e78 2048 6172 6477 6172
	0x0050:  6520 5365 7276 6572 004f 534e 616d 653d
	0x0060:  4c69 6e75 7820 342e 342e 302d 3136 392d
	0x0070:  6765 6e65 7269 6300 5573 6572 4e61 6d65
	0x0080:  3d6e 6670 0041 6765 6e74 4944 3d35 6531
	0x0090:  3565 3663 332d 3364 6164 2d34 3065 352d
	0x00a0:  3835 3164 2d63 6663 3537 6330 3230 6433
	0x00b0:  3000 5472 616e 7370 6f72 744e 616d 653d
	0x00c0:  5443 5000 506f 7274 3d33 3132 3100 5365
	0x00d0:  7276 6963 654d 616e 6167 6572 4944 3d35
	0x00e0:  6531 3565 3663 332d 3364 6164 2d34 3065
	0x00f0:  352d 3835 3164 2d63 6663 3537 6330 3230
	0x0100:  6433 302d 3000 486f 7374 3d31 3732 2e32
	0x0110:  302e 322e 3135 3300
 */

impl <'a> EthernetRx <'a> {
    pub fn new(buffer : &mut [u8;64] ) -> EthernetRx {
        EthernetRx { 
            bytes_valid: 0,
            data : buffer,
            packet_bytes : 0,
            }
    }

    pub fn be32(&self, ofs:usize) -> usize {
        ((self.data[ofs]   as usize)<<24) |
        ((self.data[ofs+1] as usize)<<16) |
        ((self.data[ofs+2] as usize)<<8) |
        ((self.data[ofs+3] as usize)<<0)
    }
    pub fn be16(&self, ofs:usize) -> usize {
        ((self.data[ofs+0] as usize)<<8) |
        ((self.data[ofs+1] as usize)<<0)
    }
    pub fn discard(&mut self, axi:&mut Axi) {
        if self.bytes_valid>0 {
            axi.rx_end_packet();
        }
        self.bytes_valid = 0;
    }

    pub fn poll(&mut self, axi:&mut Axi) -> bool {
        if self.bytes_valid>0 {true}
        else if !axi.rx_poll() {
            false
        } else {
            let size = axi.rx_start_packet() as usize;
            self.packet_bytes = size*4;
            let size = if size>12 {12} else {size};
            unsafe {
                let mut data_ptr: *mut u32  = self.data.as_mut_ptr() as *mut u32;
                for i in 0..size {
                    let rx_d = axi.rx_read_u32_raw();
                    *data_ptr = rx_d;
                    data_ptr = data_ptr.offset(1);
                }
            }
            self.bytes_valid = size*4;
            true
        }
    }

    pub fn check_dest_mac(&self) -> bool {
        if self.bytes_valid<12 {
            false
        }
        else {
            true
        }
    }
        
    pub fn is_arp_ipv4(&mut self) -> bool {
        let ethertype = self.be16(12); // 0x0806
        let arp_htype = self.be16(14); // 1 for ethernet
        let arp_ptype = self.be16(16); // 0x0800 for ipv4
        let arp_hlen  = self.data[18]; // 6 for ethernet
        let arp_plen  = self.data[19]; // 4 for ipv4
        // 20-31 source hardware
        // 32-35 source ipv4
        // 36-41 dest hardware
        // 42-45 dest ipv4
        if self.bytes_valid<46 {
            false
        } else if ethertype!=0x0860 {
            false
        } else if (arp_htype!=1) || (arp_ptype!=0x0800) || (arp_hlen!=6) || (arp_plen!=4) {
            false
        } else {
            true
        }
    }

    pub fn is_simple_ipv4(&mut self) -> bool {
        let ethertype = self.be16(12); // 0x0800
        let ipv4_vh = self.data[14];
        let ipv4_dscp_ecn = self.data[15];
        let ipv4_payload_length  = self.be16(16);
        let ipv4_identification  = self.be16(18);
        let ipv4_fragment        = self.be16(20);
        let ipv4_ttl             = self.data[22];
        let ipv4_proto           = self.data[23]; // 0x11 for UDP
        let ipv4_hdr_csum        = self.be16(24);
        let ipv4_src_ip          = self.be32(26);
        let ipv4_dest_ip         = self.be32(30);
        let ipv4_src_port        = self.be16(34); // Assumes udp/tcp - not part of IPV4 header
        let ipv4_dest_port       = self.be16(36);
        let ipv4_udp_length      = self.be16(38);
        let ipv4_udp_csum        = self.be16(40); // ones complement of source ip, dest ip, zero/protocol 0x0011, udp length (twice), source port, dest port, and udp payload (padded with 0)
        if self.bytes_valid<40 {
            false
        } else if ethertype!=0x0800 {
            false
        } else if ipv4_vh!=0x45 { // no options supported
            false
        } else if (ipv4_payload_length+14>self.packet_bytes) || (ipv4_payload_length<20) {
            false
        } else {
            true
        }
    }
        
    pub fn rx_packet(&mut self, axi:&mut Axi, data:&mut [u8]) -> bool {
        if self.bytes_valid==0 {
            false
        } else {
            false
                /*
            let size = axi.rx_start_packet() as usize;
            self.packet_bytes = size*4;
            let size = if size>16 {16} else {size};
            for i in 0..self.bytes_valid {
                data
                self.data[i*4+0] = (rx_d >> 0) as u8;
                self.data[i*4+1] = (rx_d >> 8) as u8;
                self.data[i*4+2] = (rx_d >>16) as u8;
                self.data[i*4+3] = (rx_d >>24) as u8;
            }
            self.bytes_valid = size*4;
            true*/
        }
        
    }

}
