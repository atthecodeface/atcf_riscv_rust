extern crate riscv_base;
use riscv_base::axi4s::Axi;

pub struct EthernetTx <'a> {
    data : &'a mut [u8;64],
    bytes_valid : usize,
    packet_bytes : usize,
}

impl <'a> EthernetTx <'a> {
    pub fn new(buffer : &mut [u8;64] ) -> EthernetTx {
        EthernetTx { 
            bytes_valid: 0,
            data : buffer,
            packet_bytes : 0,
            }
    }

    pub fn set_src(&mut self, mac:(u32, u16)) {
        self.data[6]  = (mac.0>>24) as u8;
        self.data[7]  = (mac.0>>16) as u8;
        self.data[8]  = (mac.0>> 8) as u8;
        self.data[9]  = (mac.0>> 0) as u8;
        self.data[10] = (mac.1>> 8) as u8;
        self.data[11] = (mac.1>> 0) as u8;
    }
    // pub fn set_u8(&mut self, ofs:usize, data:u8) {
    //     self.data[ofs] = data;
    // }
    pub fn set_be32(&mut self, ofs:usize, data:u32) {
        self.data[ofs  ]  = (data>>24) as u8;
        self.data[ofs+1]  = (data>>16) as u8;
        self.data[ofs+2]  = (data>> 8) as u8;
        self.data[ofs+3]  = (data>> 0) as u8;
    }
    pub fn set_be16(&mut self, ofs:usize, data:u16) {
        self.data[ofs  ]  = (data>> 8) as u8;
        self.data[ofs+1]  = (data>> 0) as u8;
    }
    pub fn copy_bytes(&mut self, ofs:usize, src:&[u8]) {
        let size = src.len();
        for i in 0..size {
            self.data[ofs+i] = src[i];
        }
    }

    pub fn transmit(&mut self, axi:&mut Axi) {
        axi.tx_start_packet();
        if self.bytes_valid<64 {self.bytes_valid = 64; }
        let size = (self.bytes_valid+3)>>2;
        unsafe {
            let mut data_ptr: *mut u32  = self.data.as_mut_ptr() as *mut u32;
            for _ in 0..size {
                let tx_d = *data_ptr;
                axi.tx_write_u32_raw(tx_d);
                data_ptr = data_ptr.offset(1);
            }
        }
        axi.tx_send_packet_raw(self.bytes_valid as u32);
        self.bytes_valid = 0;
    }

    pub fn transmit_with_data(&mut self, axi:&mut Axi, data:&[u8], data_size:usize ) {
        axi.tx_start_packet();
        let last_word = self.bytes_valid>>2;
        unsafe {
            let mut data_ptr: *mut u32  = self.data.as_mut_ptr() as *mut u32;
            for _ in 0..last_word {
                let tx_d = *data_ptr;
                axi.tx_write_u32_raw(tx_d);
                // riscv_base::dprintf::write4(0,(0x87,tx_d,0xffffffff,0));
                data_ptr = data_ptr.offset(1);
            }
        }
        let spare_bytes = self.bytes_valid&3;
        let mut spare_data = 0u32;
        let mut spare_bytes_valid = spare_bytes;
        for i in 0..spare_bytes {
            spare_data = spare_data | ((self.data[(last_word<<2)+i] as u32) << (8*i));
        }
        for i in 0..data_size {
            if spare_bytes_valid==4 {
                axi.tx_write_u32_raw(spare_data);
                // riscv_base::dprintf::write4(0,(0x87,spare_data,0xffffffff,0));
                spare_data = 0;
                spare_bytes_valid = 0;
            }
            spare_data = spare_data | ((data[i] as u32) << (8*spare_bytes_valid));
            spare_bytes_valid = spare_bytes_valid + 1;
        }
        axi.tx_write_u32_raw(spare_data);
        // riscv_base::dprintf::write4(0,(0x87,spare_data,0xffffffff,0));
        let total_bytes = self.bytes_valid + data_size;
        let total_bytes = if total_bytes<64 {64} else {total_bytes};
        axi.tx_send_packet_raw(total_bytes as u32);
        self.bytes_valid = 0;
    }

    pub fn create_arp_reply(&mut self, eth_rx:&EthernetRx, mac:(u32, u16) )  {
        // copy src mac as dest mac
        self.copy_bytes(0, &eth_rx.data[6..12]);
        // copy ethertype, arp_htype/ptypr/hlen/plen
        self.copy_bytes(12, &eth_rx.data[12..20]);
        // Make it an ARP reply
        self.set_be16(20, 2);
        // copy source hardware/ipv4 as dest hardware/ipv4
        self.copy_bytes(32, &eth_rx.data[22..32]);
        // copy dest ipv4 as source ipv4
        self.copy_bytes(28, &eth_rx.data[38..42]);
        // set source hw
        self.set_be32(22, mac.0);
        self.set_be16(26, mac.1);
        self.bytes_valid = 42;
    }

    pub fn create_udp_ipv4_pkt_hdr(&mut self, src_port:u16, src_ipv4:u32, dest_mac:(u32, u16), dest_ipv4:u32, dest_port:u16, udp_payload_bytes:usize )  {
        self.set_be32(0, dest_mac.0);
        self.set_be16(4, dest_mac.1);
        self.set_be16(12, 0x0800u16); // ethertype
        let ipv4_payload_length = 20 + udp_payload_bytes + 8; // 8 is 2xu16 ports + u16 length + u16 csum
        let ipv4_identification  = 0;
        let ipv4_version         = 0x4500;
        let ipv4_fragment        = 0;
        let ipv4_ttl_proto       = 0x4011; // 0x11 for UDP
        let ipv4_src_ip  = src_ipv4 as usize;
        let ipv4_dest_ip = dest_ipv4 as usize;
        let ipv4_hdr_csum        = ipv4_version + ipv4_payload_length + ipv4_identification + ipv4_fragment + ipv4_ttl_proto + (ipv4_dest_ip >> 16) + (ipv4_dest_ip & 0xffff) + (ipv4_src_ip >> 16) + (ipv4_src_ip & 0xffff);
        let ipv4_hdr_csum        = (ipv4_hdr_csum & 0xffff) + (ipv4_hdr_csum>>16); // max of 0x1fffe
        let ipv4_hdr_csum        = (ipv4_hdr_csum & 0xffff) + (ipv4_hdr_csum>>16); // max of 0xffff
        let ipv4_hdr_csum        = (ipv4_hdr_csum ^ 0xffff);
        self.set_be16(14, ipv4_version as u16);
        self.set_be16(16, ipv4_payload_length as u16);
        self.set_be16(18, ipv4_identification as u16);
        self.set_be16(20, ipv4_fragment as u16);
        self.set_be16(22, ipv4_ttl_proto as u16);
        self.set_be16(24, ipv4_hdr_csum as u16);
        self.set_be32(26, ipv4_src_ip as u32);
        self.set_be32(30, ipv4_dest_ip as u32);
        self.set_be16(34, src_port );
        self.set_be16(36, dest_port );
        self.set_be16(38, (udp_payload_bytes+8) as u16);
        self.set_be16(40, 0); // No UDP checksum as yet // ones complement of source ip, dest ip, zero/protocol 0x0011, udp length (twice), source port, dest port, and udp payload (padded with 0)
        self.bytes_valid = 42;
    }

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
            let _ = axi.rx_read_u32_raw(); // drop user
            let size = size-1;
            self.packet_bytes = size*4;
            let size = if size>12 {12} else {size};
            unsafe {
                let mut data_ptr: *mut u32  = self.data.as_mut_ptr() as *mut u32;
                for _ in 0..size {
                    let rx_d = axi.rx_read_u32_raw();
                    *data_ptr = rx_d;
                    data_ptr = data_ptr.offset(1);
                }
            }
            self.bytes_valid = size*4;
            true
        }
    }

    pub fn check_dest_mac(&self, mac:(u32,u16)) -> bool {
        if self.bytes_valid<12 {
            false
        } else {
            let mac0 = self.be32(0) as u32;
            let mac1 = self.be16(4) as u16;
 riscv_base::dprintf::wait();
 riscv_base::dprintf::write4(0,(0x4d616387, mac0 as u32,
                             (0x830000ffu32 | ((mac1 as u32)<<8)),
                             0 ));
            if (mac0==0xffffffff) && (mac1==0xffff) {
                true
            } else if (mac0==mac.0) && (mac1==mac.1) {
                true
            } else {
                false
            }
        }
    }
        
    pub fn is_arp_ipv4(&mut self, ip:u32) -> bool {
        let ethertype = self.be16(12); // 0x0806
        let arp_htype = self.be16(14); // 1 for ethernet
        let arp_ptype = self.be16(16); // 0x0800 for ipv4
        let arp_hlen  = self.data[18]; // 6 for ethernet
        let arp_plen  = self.data[19]; // 4 for ipv4
        let dest_ip   = self.be32(38) as u32; // our ip if for us
        // 22-27 source hardware
        // 28-31 source ipv4
        // 32-37 dest hardware
        // 38-41 dest ipv4
        if self.bytes_valid<46 {
            false
        } else if ethertype!=0x0806 {
            false
        } else if (arp_htype!=1) || (arp_ptype!=0x0800) || (arp_hlen!=6) || (arp_plen!=4) {
            false
        } else if dest_ip!=ip {
            false
        } else {
            true
        }
    }

    pub fn is_simple_ipv4(&self, ip:u32) -> bool {
        let ethertype = self.be16(12); // 0x0800
        let ipv4_vh = self.data[14];
        // let ipv4_dscp_ecn = self.data[15];
        let ipv4_payload_length  = self.be16(16);
        // let ipv4_identification  = self.be16(18);
        // let ipv4_fragment        = self.be16(20);
        // let ipv4_ttl             = self.data[22];
        // let ipv4_proto           = self.data[23]; // 0x11 for UDP
        // let ipv4_hdr_csum        = self.be16(24);
        // let ipv4_src_ip          = self.be32(26) as u32;
        let ipv4_dest_ip         = self.be32(30) as u32;
        // let ipv4_src_port        = self.be16(34); // Assumes udp/tcp - not part of IPV4 header
        // let ipv4_dest_port       = self.be16(36);
        // let ipv4_udp_length      = self.be16(38);
        // let ipv4_udp_csum        = self.be16(40); // ones complement of source ip, dest ip, zero/protocol 0x0011, udp length (twice), source port, dest port, and udp payload (padded with 0)
        if self.bytes_valid<40 {
            false
        } else if ethertype!=0x0800 {
            false
        } else if ipv4_vh!=0x45 { // no options supported
            false
        } else if (ipv4_payload_length+14>self.packet_bytes) || (ipv4_payload_length<20) {
            false
        } else if ip != ipv4_dest_ip {
            false
        } else {
            true
        }
    }
        
    pub fn is_udp(&self, port:u16) -> bool { // assumes it is already ipv4 (and simple)
        let ipv4_proto           = self.data[23]; // 0x11 for UDP
        let ipv4_dest_port       = self.be16(36) as u16;
        // let ipv4_udp_csum        = self.be16(40); // ones complement of source ip, dest ip, zero/protocol 0x0011, udp length (twice), source port, dest port, and udp payload (padded with 0)
        // if udp && dest_port==client_port && udp csum okay && long enough
        if ipv4_proto!=0x11 {
            false
        } else if ipv4_dest_port!=port {
            false
        } else {
            true
        }
    }
        
    pub fn ipv4_src_ip(&self) -> u32 {
        let ipv4_src_ip          = self.be32(26) as u32;
        ipv4_src_ip
        }

    pub fn ipv4_src_port(&self) -> u16 {
        let ipv4_src_port        = self.be16(34) as u16;
        ipv4_src_port
        }
        
    pub fn udp_payload_length(&self) -> usize {
        let ipv4_udp_length      = self.be16(38) as usize;
        ipv4_udp_length - 8 // as UDP field includes ports, length and csum
        }
        
    pub fn eth_src_mac(&self) -> (u32, u16) {
        (self.be32(6) as u32,self.be16(10) as u16)
        }
        
    pub fn copy_udp_payload(&self, axi:&mut Axi, data:&mut [u8]) -> usize {
        // payload starts at byte 42
        if self.bytes_valid<42 {
            0
        } else {
            let from_buf = self.bytes_valid-42;
            for i in 0..from_buf {
                data[i] = self.data[42+i];
            }
            let payload_len = self.udp_payload_length();
            let data_len = data.len();
            let num_bytes = if payload_len<data_len {payload_len} else {data_len}; // 516
            let words = (num_bytes - from_buf)>>2; // presumably 0x204>>2 = 0x81 = 129
            // riscv_base::dprintf::write4(0,(0x87,num_bytes as u32,0xffffffff,0)); // 0x20a (expected to be 0x204 I think)
            // riscv_base::dprintf::write4(0,(0x87,self.bytes_valid as u32,0xffffffff,0)); // 0x30
            for i in 0..words {
                let rx_d = axi.rx_read_u32_raw();
                let ofs = from_buf + (i<<2);
                data[ofs+0] = (rx_d >> 0) as u8;
                data[ofs+1] = (rx_d >> 8) as u8;
                data[ofs+2] = (rx_d >>16) as u8;
                data[ofs+3] = (rx_d >>24) as u8;
            }
            // unsafe {
            //     let mut data_ptr: *mut u32  = data.as_mut_ptr() as *mut u32;
            //     for _ in 0..16 {
            //         let d = *data_ptr;
            //         riscv_base::dprintf::write4(0,(0x87,d,0xffffffff,0));
            //         data_ptr = data_ptr.offset(1);
            //     }
            // }
            num_bytes
        }
        
    }

}
