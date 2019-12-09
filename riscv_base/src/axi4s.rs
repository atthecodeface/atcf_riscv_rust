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

pub enum AxiError {
    RxOverflow,
}
pub struct Axi {
    tx_ptr : u32,
    rx_ptr : u32,
    sram_size : u32,
    next_rx_ptr : u32,
    tx_size : u32,
    tx_tail : u32,
    tx_tail_size : u32,
    rx_size : u32,
    rx_tail : u32,
    rx_tail_size: u32
}
impl Axi {
    pub fn new(sram_size:u32) -> Axi {
        Axi { 
            tx_ptr: 0, 
            rx_ptr : 0, 
            sram_size, 
            next_rx_ptr:0,
            tx_size: 0, 
            tx_tail : 0,
            tx_tail_size : 0,
            rx_size : 0,
            rx_tail : 0,
            rx_tail_size : 0,
            }
    }
    pub fn reset(&mut self) {
        set_tx_ptr(0);
        write_tx_data(0);
        write_tx_config(self.sram_size);
        write_rx_config(self.sram_size);
        self.tx_ptr = 0;
        self.rx_ptr = 0;
        self.next_rx_ptr = 0;
        self.rx_tail = 0;
        self.rx_tail_size = 0;
    }

    // Preprare the tx buffer for new data
    pub fn tx_start_packet(&mut self) {
        set_tx_ptr(self.tx_ptr);
        write_tx_data_inc(0); // status
        write_tx_data_inc(0); // user not used by GbE at present
        self.tx_size = 0;
        self.tx_tail_size = 0;
        self.tx_tail = 0;
    }

    // Write a single word into the tx buffer
    pub fn tx_write_u32(&mut self, data: u32) {
        if self.tx_tail_size == 0 {
            write_tx_data_inc(data);
        } else {
            let mut word = self.tx_tail;
            word |= data >> (self.tx_tail_size * 8);
            write_tx_data_inc(word);
            self.tx_tail = data << (24 - self.tx_tail_size * 8);
        }
        self.tx_size += 1;
    }

    // Write the byte array into the tx buffer
    pub fn tx_write_data_le(&mut self, data: &[u8]) {

        let mut word = self.tx_tail;
        let mut pack_count = self.tx_tail_size;

        for i in 0..data.len() {
            if pack_count == 4 {
                write_tx_data_inc(word);
                self.tx_size += 1;
                word = 0;
                pack_count = 0;
            }
            word = word << 8 | (data[i] as u32);
            pack_count += 1;
        }

        self.tx_tail = word;
        self.tx_tail_size = pack_count;
    }

    // Write the byte array into the tx buffer
    pub fn tx_write_data(&mut self, data: &[u8]) {

        let mut word = self.tx_tail;
        let mut pack_count = self.tx_tail_size;

        for i in 0..data.len() {
            if pack_count == 4 {
                write_tx_data_inc(word);
                self.tx_size += 1;
                word = 0;
                pack_count = 0;
            }

            word |= (data[i] as u32) << (24 - pack_count * 8);
            pack_count += 1;
        }

        self.tx_tail = word;
        self.tx_tail_size = pack_count;
    }

    // Mark the packet as ready for submission 
    pub fn tx_send_packet(&mut self) {
        if self.tx_tail_size != 0 {
            write_tx_data_inc(self.tx_tail);
            self.tx_size += 1;
        }
        write_tx_data(0); 
        set_tx_ptr(self.tx_ptr);
        write_tx_data(self.tx_size<<2); // number of bytes in packet
        self.tx_ptr = (self.tx_ptr + 2 + self.tx_size);// % self.sram_size;
        if self.tx_ptr > self.sram_size {
            self.tx_ptr -= self.sram_size;
        }

        self.tx_tail = 0;
        self.tx_tail_size = 0;
        self.tx_size = 0;

    }

    pub fn tx_write_u32_raw(&mut self, data:u32) {
        write_tx_data_inc(data);
    }

    // Mark the packet as ready for submission 
    pub fn tx_send_packet_raw(&mut self, byte_size:u32) {
        let word_size = (byte_size+3)>>2;
        write_tx_data(0); 
        set_tx_ptr(self.tx_ptr);
        write_tx_data(byte_size);
        self.tx_ptr = (self.tx_ptr + 2 + word_size);
        if self.tx_ptr > self.sram_size {
            self.tx_ptr -= self.sram_size;
        }
    }




    // Roll the buffer back to start of failed packet
    pub fn tx_fail_packet(&mut self) {
        set_tx_ptr(self.tx_ptr);

        self.tx_tail = 0;
        self.tx_tail_size = 0;
        self.tx_size = 0;
    }

    // Poll to see of there is a packet availalbe in the rx queue
    pub fn rx_poll(&self) -> bool {
        set_rx_ptr(self.rx_ptr);
        let rx_status = read_rx_data();
        ((rx_status>>31)&1)==1
    }

    // Prepare to read a packet from buffer
    pub fn rx_start_packet(&mut self) -> u32 {
        let rx_status = read_rx_data_inc();
        let word_size = rx_status & 0x3ff;
        self.next_rx_ptr = self.rx_ptr + word_size;
        if self.next_rx_ptr > self.sram_size {
            self.next_rx_ptr -= self.sram_size;
        }
        
        self.rx_size = word_size;
        self.rx_tail = 0;
        self.rx_tail_size = 0;
        word_size - 1
    }

    pub fn rx_read_u32_raw(&mut self) -> u32 {
        read_rx_data_inc()
    }

    // Read a single word from the packet
    pub fn rx_read_u32(&mut self) -> u32 {

        if self.rx_size <= 0 {
		return 0;
        //    return Err(AxiError::RxOverflow);
        }
        if self.rx_tail_size == 0 {
            let word = read_rx_data_inc();
            self.rx_size -= 1;
            return word;//Ok(word);
        }
        else {
            let mut rslt = self.rx_tail;
            let word = read_rx_data_inc();
            rslt |= word >> (self.tx_tail_size * 8);
            self.rx_tail = word << (24 - self.rx_tail_size * 8);
            self.rx_size -= 1;
            return rslt;//Ok(rslt);
        }
    }

    // Read data from the current packet into the buffer, return the number of bytes read.
    pub fn rx_read_data(&mut self, buffer: &mut [u8]) -> usize {

        for i in 0..buffer.len() {
            if self.rx_tail_size == 0 {
                if self.rx_size == 0 {
                    return i;
                }
                self.rx_tail = read_rx_data_inc();
                self.rx_tail_size = 4;
                self.rx_size -= 1
            }
            buffer[i] = (self.rx_tail >> (24 - self.rx_tail_size * 8)) as u8;
            self.rx_tail_size -= 1;
        }
        return buffer.len();
    }

    // Read data from the current packet into the buffer in little endian, return the number of bytes read.
    pub fn rx_read_data_le(&mut self, buffer: &mut [u8]) -> usize {

        for i in 0..buffer.len() {
            if self.rx_tail_size == 0 {
                if self.rx_size == 0 {
                    return i;
                }
                self.rx_tail = read_rx_data_inc();
                self.rx_tail_size = 4;
                self.rx_size -= 1
            }
            buffer[i] = self.rx_tail as u8;
            self.rx_tail = self.rx_tail >> 8;
            self.rx_tail_size -= 1;
        }
        return buffer.len();
    }


    // Mark the packet as finished. 
    pub fn rx_end_packet(&mut self) {
	    self.rx_ptr = self.next_rx_ptr;
        set_rx_ptr(self.next_rx_ptr);
        commit_rx_ptr()
    }
}
