
//    apb_address_i2c_config      = 0,
//    apb_address_master_status   = 1,
//    apb_address_master_control  = 2,
//    apb_address_master_data     = 3,
//    apb_address_master_command  = 4,

//        case access_read_status: {
//            apb_response.prdata = 0;
//            apb_response.prdata[3;4]  = master_response.response_type;
//            apb_response.prdata[1]    = master_response.in_progress;
//            apb_response.prdata[0]    = apb_state.master_request.valid;
//        }
//        case access_read_i2c_config: {
//            apb_response.prdata = 0;
//            apb_response.prdata[4;24] = apb_state.master_conf.data_setup_delay;
//            apb_response.prdata[4;20] = apb_state.master_conf.data_hold_delay;
//            apb_response.prdata[4;16] = apb_state.master_conf.period_delay;
//            apb_response.prdata[8; 8] = apb_state.i2c_conf.period;
//            apb_response.prdata[8; 0] = apb_state.i2c_conf.divider;
//        }
//        case access_read_master_control: {
//            apb_response.prdata = 0;
//            apb_response.prdata[9]     = apb_state.master_request.valid;
//            apb_response.prdata[8]     = apb_state.master_request.cont;
//            apb_response.prdata[ 3; 4] = apb_state.master_request.num_in;
//            apb_response.prdata[ 3; 0] = apb_state.master_request.num_out;
//        }
//        case access_read_master_data: {
//            apb_response.prdata = master_response.data;
//        }

//        case access_write_master_control: {
//            apb_state.master_request.valid       <= apb_request.pwdata[9];
//            apb_state.master_request.cont        <= apb_request.pwdata[8];
//            apb_state.master_request.num_in      <= apb_request.pwdata[ 3; 4];
//            apb_state.master_request.num_out     <= apb_request.pwdata[ 3; 0];
//        }


pub fn write_i2c_config(data:u32) {
    unsafe {
        let apb_i2c_config: *mut u32 = super::minimal::APB_I2C_MASTER.offset(0);
        core::ptr::write_volatile(apb_i2c_config, data);
    }
}

pub fn read_status() -> u32 {
    unsafe {
        let apb_status: *const u32 = super::minimal::APB_I2C_MASTER.offset(1);
        core::ptr::read_volatile(apb_status)
    }
}

pub fn is_busy() -> bool {
    (read_status()&3)!=0
}

pub fn wait() {
    while is_busy() {
        unsafe {super::sleep(1000);}
    };
}

pub fn i2c_response() -> u32 {
    (read_status()>>4)&7
}

pub fn write_data(data:u32) {
    unsafe {
        let apb_master_data: *mut u32 = super::minimal::APB_I2C_MASTER.offset(3);
        core::ptr::write_volatile(apb_master_data, data);
    };
}

pub fn write_control(control:u32) {
    unsafe {
        let apb_master_control: *mut u32 = super::minimal::APB_I2C_MASTER.offset(2);
        core::ptr::write_volatile(apb_master_control, control);
    };
}

pub fn read_data() -> u32 {
    unsafe {
        let apb_master_data: *const u32 = super::minimal::APB_I2C_MASTER.offset(3);
        core::ptr::read_volatile(apb_master_data)
    }
}

pub fn exec(num_out:u32, num_in:u32, cont:bool, data_in:u32) -> (bool, u32) {
    write_data(data_in);
    let control = 0x200 | (if cont {0x100} else {0}) | (num_in<<4) | (num_out<<0);
    write_control(control);
    wait();
    if i2c_response()!=0 { (false, 0) } else {
       (true, read_data())
    }
}

