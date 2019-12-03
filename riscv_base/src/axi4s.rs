//    apb_address_rx_config      = 0   "Receive configuration",
//    apb_address_rx_data_ptr    = 1   "Receive data pointer",
//    apb_address_rx_data        = 2   "Receive data",
//    apb_address_rx_data_next   = 3   "Receive data and move on",
//    apb_address_rx_commit      = 4   "Mark current receive data pointer as head of read",
//    apb_address_tx_config      = 8   "Transmit configuration",
//    apb_address_tx_data_ptr    = 9   "Transmit data pointer",
//    apb_address_tx_data        = 10  "Transmit data",
//    apb_address_tx_data_next   = 11  "Transmit data and move on",

// In the current version of the hardware - very preliminary:
//
// Rx data for a packet is
//  32-bit status;
//    0 for no packet ready (particularly top bit 0)
//    {1b1, 00..00, pkt_words[10;0]} if a packet is valid
//  Then 32-bits of AXI4S user data from the first AXI4S USER word
//  Then (pkt_words-1) * 32-bits of packet data

pub fn write_rx_config(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 0, data);
}

pub fn write_tx_config(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 8, data);
}

pub fn set_rx_ptr(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 1, data);
}

pub fn read_rx_data() -> u32 {
    super::minimal::read_dev_apb(super::minimal::APB_AXI4S, 2)
}

pub fn read_rx_data_inc() -> u32 {
    super::minimal::read_dev_apb(super::minimal::APB_AXI4S, 3)
}

pub fn commit_rx_ptr() {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 4, 0)
}

pub fn set_tx_ptr(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 9, data);
}


pub fn write_tx_data(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 10, data);
}


pub fn write_tx_data_inc(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_AXI4S, 11, data);
}

pub struct Axi {
    tx_ptr : u32,
    rx_ptr : u32,
    sram_size : u32,
    next_rx_ptr : u32,
}
impl Axi {
    pub fn new(sram_size:u32) -> Axi {
        Axi { tx_ptr: 0, rx_ptr : 0, sram_size, next_rx_ptr:0 }
    }
    pub fn reset(&mut self) {
        set_rx_ptr(0);
        set_tx_ptr(0);
        write_tx_data(0);
        write_tx_config(self.sram_size);
        write_rx_config(self.sram_size);
        self.tx_ptr = 0;
        self.rx_ptr = 0;
        self.next_rx_ptr = 0;
    }
    pub fn send_pkt(&mut self, byte_size:u32) {
        let word_size = (byte_size+3)/4;
        set_tx_ptr(self.tx_ptr);
        write_tx_data_inc(0); // status
        write_tx_data_inc(0); // user not used by GbE at present
        for i in 1..word_size {
            write_tx_data_inc(i); // 
        }
        write_tx_data_inc(0); // next packet start will be at 2+length in words
        set_tx_ptr(self.tx_ptr);
        write_tx_data_inc(byte_size); // number of bytes in packet
        self.tx_ptr = (self.tx_ptr + 2 + word_size) / self.sram_size;
    }
    pub fn rx_poll(&self) -> bool {
        let rx_status = read_rx_data();
        ((rx_status>>31)&1)==1
    }
    pub fn rx_start_packet(&mut self) -> u32 {
        let rx_status = read_rx_data();
        let word_size = rx_status & 0x3ff;
        self.next_rx_ptr = self.rx_ptr + word_size;
        word_size
    }
    pub fn rx_read_data(&self) -> u32 {
        read_rx_data_inc()
    }
    pub fn rx_end_packet(&self) {
        set_rx_ptr(self.next_rx_ptr);
        commit_rx_ptr()
    }
}
