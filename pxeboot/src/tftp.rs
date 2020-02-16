pub trait TftpSocket {
    fn reset(&mut self);
    fn has_rx_data(&self) -> bool; // return true if UDP socket has data ready
    fn can_tx_data(&self) -> bool; // return true if UDP socktet can transmit a packet
    fn rx_data(&mut self, data:&mut[u8]) -> usize; // has_rx_data() has returned true; copy as much data as you have (up to data.len()) to data
    fn tx_data(&mut self, data:&[u8]) -> usize;
}

enum TftpState {
    Unconnected,
    Block(u32),
    Ack(u32,bool),
    Finished,
}

pub enum TftpEvent {
    Connect,
    Data(usize,usize),
    Error,
    Idle,
}

pub struct Tftp {
    state    : TftpState,
    timeout  : u32,
}

const DATA_TIMEOUT : u32 = 1000000;

fn read_u16_be(data:&[u8]) -> usize {
    (data[1] as usize) |
    ((data[0] as usize)<<8)
}


impl Tftp {
    pub fn new() -> Tftp {
        Tftp {state:TftpState::Unconnected, timeout:0}
    }
    pub fn reset(&mut self) {
        //println!("tftp reset");
        self.state = TftpState::Unconnected;
        self.timeout = 1000;
    }
    fn handle_rx_data( &mut self, socket:&mut TftpSocket, data:&mut [u8]) -> TftpEvent {
        let size = socket.rx_data(data);
        match self.state {
            TftpState::Unconnected => { // received data when unconnected - drop it
                //println!("handle rx data when unconnected");
                self.timeout = DATA_TIMEOUT;
                TftpEvent::Idle
            },
            TftpState::Block(expected) => {
                let op     = read_u16_be(&data[0..2]);
                let actual = read_u16_be(&data[2..4]) as u32;
                //println!("handle rx data {:?} {:?} {:?} when expecting block {:?}",size,op,actual,expected);
                if (size>4) && (size<=516) && (op==3) {
                    if actual==expected {
                        //println!("Now in ack {:?}",actual);
                        self.timeout = 1;
                        self.state = TftpState::Ack(actual,size<516);
                        TftpEvent::Data(4,size-4)
                    } else if actual==(expected-1) {
                        //println!("Got a duplicate {:?}",actual);
                        TftpEvent::Idle
                    } else {
                        //println!("Ack for unexpected packet {:?} when expecting block {:?}",actual,expected,);
                        TftpEvent::Error
                    }
                } else {
                    //println!("Bad size or op {:?} {:?} when expecting block {:?}",size,op,expected);
                    TftpEvent::Error
                }
            },
            TftpState::Ack(_expected,_finished) => { // received data when about to send an ack - drop it
                //println!("handle rx data when about to send ack for block {:?}",expected);
                TftpEvent::Idle
            },
            TftpState::Finished => { // received data when done - drop it
                //println!("handle rx data when finished");
                TftpEvent::Idle
            },
        }
    }
    fn send_request(&mut self, socket:&mut TftpSocket) -> TftpEvent {
        //println!("send file get request");
        socket.reset();
        let mut tx_buf : [u8;32] = [0; 32];
        tx_buf[0] = 0;
        tx_buf[1] = 1;
        tx_buf[2..8].copy_from_slice(b"banana");
        tx_buf[9..14].copy_from_slice(b"octet");
        socket.tx_data(&tx_buf[..18]);
        self.timeout = DATA_TIMEOUT;
        self.state = TftpState::Block(1);
        TftpEvent::Connect
    }
    fn send_ack(&mut self, socket:&mut TftpSocket, block:u32, finished:bool) -> TftpEvent {
        //println!("send ack for block {:?}",block);
        let mut tx_buf : [u8;32] = [0; 32];
        tx_buf[0] = 0;
        tx_buf[1] = 4;
        tx_buf[2] = (block>>8) as u8;
        tx_buf[3] = block as u8;
        socket.tx_data(&tx_buf[..4]);
        self.timeout = DATA_TIMEOUT;
        if finished {
            self.state = TftpState::Finished;
        } else {
            self.state = TftpState::Block(block+1);
        }
        TftpEvent::Idle
    }
    fn handle_timeout(&mut self, socket:&mut TftpSocket) -> TftpEvent {
        match self.state {
            TftpState::Unconnected     => {
                self.send_request(socket)
            },
            TftpState::Block(expected) => {
                if expected==1 {
                    self.send_request(socket)
                } else {
                    self.state = TftpState::Ack(expected-1,false); // Force a retry on the ack
                    TftpEvent::Idle
                }
            },
            TftpState::Ack(expected,finished) => {
                self.send_ack(socket,expected,finished)
            },
            TftpState::Finished => {
                self.timeout = DATA_TIMEOUT;
                TftpEvent::Idle
            },
        }
    }
    pub fn poll(&mut self, socket:&mut TftpSocket) -> bool {
        if socket.has_rx_data() {
            //println!("has rx data with timeout {:?}", self.timeout);
            true
        }
        else if self.timeout==0 {
            //println!("timedout");
            true
        }
        else {
            self.timeout = self.timeout-1;
            false
        }
    }
    pub fn get_event(&mut self, socket:&mut TftpSocket, data:&mut [u8]) -> TftpEvent {
        if socket.has_rx_data() {
            self.handle_rx_data(socket, data)
        } else if self.timeout==0 {
            self.handle_timeout(socket, )
        } else {
            TftpEvent::Idle
        }
    }
}
