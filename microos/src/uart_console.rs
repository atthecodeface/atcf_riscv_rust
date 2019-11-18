extern crate riscv_base;

use riscv_base::uart::{status, tx, rx};

#[derive(Default)]
pub struct Buffer<T: Copy> {
pub        count : usize,
pub        read  : usize,
pub        write : usize,
pub        size  : usize,
pub        ready : bool,
pub        hack:T
}

impl <T:Copy> Buffer<T> {
     fn reset(&mut self) {
        self.count=0;
        self.read=0;
        self.write=0;
        self.ready = false;
     }
     fn mark_ready(&mut self) {
         self.ready = true;
     }
     fn is_ready(&self) -> bool {
        self.ready || (self.count==self.size)
     }
     fn is_empty(&self) -> bool {
        self.count==0
     }
     fn is_full(&self) -> bool {
        self.count==self.size
     }
     fn pop(&mut self, buffer:&[T]) -> T {
        let data = buffer[self.read];
        self.read  = self.read+1;
        if self.read==self.size { self.read = 0; }
        self.count = self.count-1;
        data
     }
     fn push(&mut self, buffer:&mut [T], data:T) {
         buffer[self.write] = data;
        self.write = self.write+1;
        if self.write==self.size { self.write = 0; }
        self.count = self.count+1;
     }
}

pub struct Console {
pub    tx_ptrs   : Buffer<u8>,
pub    rx_ptrs   : Buffer<u8>,
pub    tx_buffer : [u8;64],
pub    rx_buffer : [u8;64],
}

impl Console {
    fn tx_buffer_empty(&self) -> bool {
        self.tx_ptrs.is_empty()
    }

    fn tx_buffer_full(&self) -> bool {
        self.tx_ptrs.is_full()
    }

    fn tx_buffer_pop(&mut self) -> u8 {
        let data = self.tx_ptrs.pop(&self.tx_buffer);
        data
    }
    pub fn tx_buffer_push(&mut self, data:u8) {
    self.tx_ptrs.push(&mut self.tx_buffer, data)
    }
    pub fn tx_buffer_push_buffer(&mut self, data:&[u8]) {
        for &d in data {
            self.tx_buffer_push(d);
        }
    }

    pub fn get_rx_buffer(&self) -> &[u8] {
        &self.rx_buffer[0..self.rx_ptrs.count]
    }

    pub fn rx_buffer_reset(&mut self) {
    self.rx_ptrs.reset()
    }

    pub fn rx_buffer_ready(&self) -> bool {
    self.rx_ptrs.is_ready()
    }

    pub fn rx_buffer_empty(&self) -> bool {
    self.rx_ptrs.is_empty()
    }

    fn rx_buffer_full(&self) -> bool {
    self.rx_ptrs.is_full()
    }

    pub fn rx_buffer_pop(&mut self) -> u8 {
    let data = self.rx_ptrs.pop(&self.rx_buffer);
    data
    }

    fn rx_buffer_push(&mut self, data:u8) {
    if data<32 { self.rx_ptrs.mark_ready();
    } else {
        self.rx_ptrs.push(&mut self.rx_buffer, data)
        }
    }
}

pub fn poll(console:&mut Console) -> bool {
    let uart_status = status();
    let mut work_done = false;
    if !console.tx_buffer_empty() {
        if (uart_status&0x100)==0 {
           let data = console.tx_buffer_pop() as u32;
           tx(data);
           work_done = true;
        }
    }
    if (uart_status&0x10000)!=0 {
       let data = rx() as u8;
       if console.rx_buffer_full() {
       } else {
         console.rx_buffer_push(data);
       }
       work_done = true;
    }
    work_done
}
