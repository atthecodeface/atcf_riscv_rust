extern crate pxeboot;

pub struct SubLoader {
}

impl SubLoader {
     pub fn new() -> SubLoader {
         SubLoader {}
     }
}

impl pxeboot::loader::SubLoader for SubLoader {
    fn copy_memory(&self, source:&[u8], dest:u32) -> bool {
      riscv_base::dprintf::write4(0,(0x4d454d87,dest,0xffffffff,0));
    /*
            unsafe {
                let mut data_ptr: *const u32  = source.as_ptr() as *const u32;
                for _ in 0..16 {
                    let d = *data_ptr;
                    riscv_base::dprintf::write4(0,(0x87,d,0xffffffff,0));
                    data_ptr = data_ptr.offset(1);
                }
            }
        //println!("Copy memory to {:?}", dest);
  */
        unsafe {
            let mut dest_ptr: *mut u8 = dest as *mut u8;
            for i in 0..source.len() {
                    *dest_ptr = source[i];
                    dest_ptr = dest_ptr.offset(1);
            }
            }
        true
    }
    fn execute(&self, address:u32) {
        riscv_base::dprintf::write4(0,(0x45584587,address,0xffffffff,0));
        unsafe {riscv_base::exec_at(address);}
        //println!("Execute at {:?}", address);
        loop {}
    }
        
}
