//    apb_address_config      = 0,
//    apb_address_trigger     = 1,
//    apb_address_mask        = 2,
//    apb_address_compare     = 3,
//    apb_address_mux_control = 4,
//    apb_address_trace_data  = 8,

fn write_mux_control(data:u32) {
    super::minimal::write_dev_apb(super::minimal::APB_ANALYZER, 4, data);
}

pub fn read_status() -> u32 {
    super::minimal::read_dev_apb(super::minimal::APB_ANALYZER, 0)
}

pub fn enable_source(select:u32, mux_control:u32, nybbles:u32) {
    write_mux_control(0);
    while {(read_status() & (1<<8))!=0} {}
    write_mux_control(1 | (select<<8));
    while {(read_status() & (1<<8))==0} {}
    write_mux_control(3 | (nybbles<<4) | (mux_control<<8));
}

