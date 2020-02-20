use core::marker::PhantomData;

pub trait SubLoader {
    fn copy_memory(&self, source:&[u8], dest:u32) -> bool;
    fn execute(&self, address:u32);
}

pub struct Loader <'a, T:SubLoader> {
    pub    data_buffer : &'a mut [u8;1024],
    bytes_valid        : usize,
    phantom: PhantomData<T>,
}

fn read_u16_le(data:&[u8]) -> usize {
    (data[0] as usize) |
    ((data[1] as usize)<<8)
}

fn read_u32_le(data:&[u8]) -> usize {
    (data[0] as usize) |
    ((data[1] as usize)<<8) |
    ((data[2] as usize)<<16) |
    ((data[3] as usize)<<24)
}

impl <'a, T:SubLoader> Loader <'a, T> {
    pub fn new(data_buffer : &'a mut [u8;1024]) -> Loader<T> {
        Loader {data_buffer, bytes_valid:0, phantom:PhantomData}
    }
    pub fn reset(&mut self) {
        self.bytes_valid = 0;
    }
    pub fn is_empty(self) -> bool {
        (self.bytes_valid == 0)
    }
    fn parse_header(&self) -> (usize,usize,u32) {
        let op      = read_u16_le(&self.data_buffer[0..2]);
        let length  = read_u16_le(&self.data_buffer[2..4]);
        let address = read_u32_le(&self.data_buffer[4..8]) as u32;
        //println!("Parse header {:?} {:?} {:?} {:?}",op,length,address,self.bytes_valid);
        (op, length, address)
    }
    fn is_block_ready(&self) -> bool {
        let (_, length, _) = self.parse_header();
        if (self.bytes_valid<8) || (self.bytes_valid<length+8) {return false;}
        return true;
    }
    fn handle_ready_block(&mut self, loader:&T) -> bool {
        let (op, length, address) = self.parse_header();
        if op==1 {
            let block_end = 8+(length as usize);
            let next_bytes_valid = self.bytes_valid - (length+8);
            if !loader.copy_memory(&self.data_buffer[8..block_end], address) {return false;}
            //println!("Moving data length {:?} back by {:?}",next_bytes_valid,block_end);
            if next_bytes_valid>0 {
                for x in 0..next_bytes_valid {
                    self.data_buffer[x] = self.data_buffer[x+block_end];
                }
            }
            self.bytes_valid = next_bytes_valid;
            let _x = self.parse_header();
            return true;
        } else if op==2 {
            loader.execute(address);
        }
        return false;
    }
    pub fn rx_data(&mut self, loader:&T, data:&[u8], size:usize) -> bool {
        while self.is_block_ready() {
            if !self.handle_ready_block(loader) {return false;}
        }
        let ofs = self.bytes_valid;
        if self.bytes_valid + size > 1024 {return false;}
        //println!("Adding in data length {:?} at {:?}",size,self.bytes_valid);
        self.bytes_valid = self.bytes_valid + size;
        for x in 0..size {
            self.data_buffer[x+ofs] = data[x];
        }
        while self.is_block_ready() {
            if !self.handle_ready_block(loader) {return false;}
        }
        true
    }
}
