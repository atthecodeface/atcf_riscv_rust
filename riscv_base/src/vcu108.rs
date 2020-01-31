// ADV7511 in XCVU108 connects data bits [16;8] and hence is 16-bit (INPUT ID=1, style=1)
// This requires Cb/Y, Cr/Y in successive cycles
// I2C address is 0x72
// Out of reset (when powers up)
// I2c register 0x41[1;6] = power down (must only be cleared when HPD is high)
// I2C register 0x98[8;0] = 0x03
// I2C register 0x9a[7;1] = 0x70
// I2C register 0x9c[7;0] = 0x30
// I2C register 0x9d[2;0] = 0x01
// I2C register 0xa2[8;0] = 0xa4
// I2C register 0xa3[8;0] = 0xa4
// I2C register 0xe0[8;0] = 0xd0
// I2c register 0xaf[1;1] = output is HDMI (not DVI)
// Input style
// I2C register 0xf9[8;0] = 0
// I2C register 0x15[4;0] = 1 (input id)  (other bits 0)
// I2C register 0x16[2;4] = 2b11 (8 bit per channel)
// I2C register 0x16[2;2] = 2b10 (style 1)  (other bits 0)
// I2C register 0x48[2;3] = 01 (right justified) (other bits 0)
// 1080p-60 is 1920x1080
// pixel clock 148.5MHz = 900MHz/6 almost
// line time is 14.8us
// frame time is 16666us
// horizontal front porch/sync/back porch/active = 88/44/148/1920 +ve sync
// vertical   front porch/sync/back porch/active = 4/5/36/1080 +ve sync

const ADV7511_INIT : [u32; 15] = [ 0xc0d6, 0x1041, 0x0398, 0xe09a, 0x309c, 0x619d, 0xa4a2, 0xa4a3, 0xd0e0, 0x00f9, 0x0115, 0x3416, 0x0848, 0x02af, 0x0217 ];
pub fn configure_adv7511() {
    // for 100MHz clock the divider can be 10, and period 10 (i.e. 10MHz I2C pin sampling, 1us period for master transitions)
    // period_delay of 10 gives 50kHz I2C interface - give a hold of 6 and setup of 4 to split period for when SDA changes
    super::i2c_master::write_i2c_config(0x046a0a0a);
    // Disable 4-port I2C expander
    super::i2c_master::exec(2, 0, false, (0x0000)|(0x75<<1)|0 );
    // Enable 8-port I2C expander to talk to ADV7511 only
    super::i2c_master::exec(2, 0, false, (0x2000)|(0x74<<1)|0 );
    // Write to ADV7511 (note can set d6[2;6] to 11 to have 'HPD is always high')
    // Note 98-ae, cd-f8 are not reset with HPD
    for w in &ADV7511_INIT {
        super::i2c_master::exec(3, 0, false, (w<<8)|(0x39u32<<1)|0u32 );
    }
    super::i2c_master::exec(2, 0, true, (0x00<<8)|(0x39<<1)|0 );
    super::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
    super::i2c_master::exec(2, 0, true, (0x3c<<8)|(0x39<<1)|0 );
    super::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
    super::i2c_master::exec(2, 0, true, (0x3d<<8)|(0x39<<1)|0 );
    super::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
    super::i2c_master::exec(2, 0, true, (0x3e<<8)|(0x39<<1)|0 );
    super::i2c_master::exec(1, 1, false, (0x39<<1)|1 );
}

