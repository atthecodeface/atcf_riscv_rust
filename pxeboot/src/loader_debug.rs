use super::loader::{SubLoader};

pub struct DebugSubLoader {
}

impl SubLoader for DebugSubLoader {
    fn copy_memory(&self, source:&[u8], dest:u32) -> bool {
        //println!("Copy memory to {:?}", dest);
        true
    }
    fn execute(&self, address:u32) {
        //println!("Execute at {:?}", address);
        loop {}
    }
        
}
