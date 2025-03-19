
use crate::dev_spi::SPIWorker;
use log::info;

pub const  FLASH_BUF_SIZE : usize = 64;
pub const  SPI_FLASH_NUM  : usize = 1;

pub const  FLASH_CMD_ID              : u8 = 0x9F;
pub const  FLASH_CMD_READ            : u8 = 0x03;
pub const  FLASH_CMD_FAST_READ       : u8 = 0x0B;
pub const  FLASH_CMD_FAST_READ_DUAL  : u8 = 0x3B;
pub const  FLASH_CMD_FAST_READ_QUAD  : u8 = 0x6B;
pub const  FLASH_CMD_CONT_READ_RESET : u8 = 0xFF;
pub const  FLASH_CMD_WRITE_ENABLE    : u8 = 0x06;
pub const  FLASH_CMD_WRITE_DISABLE   : u8 = 0x04;
pub const  FLASH_CMD_READ_SFDP       : u8 = 0x5A;
pub const  FLASH_CMD_BLOCK_ERASE     : u8 = 0xD8;
pub const  FLASH_CMD_SECTOR_ERASE    : u8 = 0x20;
pub const  FLASH_CMD_PAGE_PROGRAM    : u8 = 0x02;
pub const  FLASH_CMD_READ_SR1        : u8 = 0x05;
pub const  FLASH_CMD_READ_SR2        : u8 = 0x35;
pub const  FLASH_CMD_READ_SR3        : u8 = 0x33;



pub const  SPI_FLASH_SR1_BIT_SRP0  : u32 = 7;  /* Status Register Protect 0 */
pub const  SPI_FLASH_SR1_BIT_SEC   : u32 = 6;  /* Sector/Block protect (0: 64kB blocks, 1: 4kB sectors) */
pub const  SPI_FLASH_SR1_BIT_TB    : u32 = 5;  /* Top/Bottom protect (0: top down, 1: bottom up) */
pub const  SPI_FLASH_SR1_BIT_BP2   : u32 = 4;  /* Block protect bits */
pub const  SPI_FLASH_SR1_BIT_BP1   : u32 = 3;  /* Block protect bits */
pub const  SPI_FLASH_SR1_BIT_BP0   : u32 = 2;  /* Block protect bits */
pub const  SPI_FLASH_SR1_BIT_BP    : u32 = 2;  /* Block protect bits */
pub const  SPI_FLASH_SR1_BP_MASK   : u32 = 7<<SPI_FLASH_SR1_BIT_BP;  /* Block protect mask */
pub const  SPI_FLASH_SR1_BIT_WE    : u32 = 1;  /* Write enable latch */
pub const  SPI_FLASH_SR1_BIT_BUSY  : u32 = 0;  /* Embedded operation status */

pub const  SPI_FLASH_SR2_BIT_SUS   : u32 = 7;  /* Suspend status (1: Erase / Program suspended) */
pub const  SPI_FLASH_SR2_BIT_CMP   : u32 = 6;  /* Complement protect (0: normal protection map, 1: inverted protection map) */
pub const  SPI_FLASH_SR2_BIT_LB3   : u32 = 5;  /* Security register lock bits */
pub const  SPI_FLASH_SR2_BIT_LB2   : u32 = 4;  /* Security register lock bits */
pub const  SPI_FLASH_SR2_BIT_LB1   : u32 = 3;  /* Security register lock bits */
pub const  SPI_FLASH_SR2_BIT_LB0   : u32 = 2;  /* Security register lock bits */
pub const  SPI_FLASH_SR2_BIT_LB    : u32 = 2;  /* Security register lock bits */
pub const  SPI_FLASH_SR2_LB_MASK   : u32 = 0xf<<SPI_FLASH_SR2_BIT_LB;  /* Security register lock-bit mask */
pub const  SPI_FLASH_SR2_BIT_QE    : u32 = 1;  /* Quad enable */
pub const  SPI_FLASH_SR2_BIT_SRP1  : u32 = 0;  /* Status register protect 1 */

pub const  SPI_FLASH_SR3_BIT_W6     : u32 = 6;  /* Burst wrap length */
pub const  SPI_FLASH_SR3_BIT_W5     : u32 = 5;  /* Burst wrap length */
pub const  SPI_FLASH_SR3_BIT_BWLEN  : u32 = 5;  /* Burst wrap length */
pub const  SPI_FLASH_SR3_BWLEN_MASK : u32 = 3<<SPI_FLASH_SR3_BIT_BWLEN;  /* Burst wrap length mask */
pub const  SPI_FLASH_SR3_BIT_BWE    : u32 = 4;  /* Burst wrap enable */
pub const  SPI_FLASH_SR3_BIT_LC3    : u32 = 3;  /* Latency control bits */
pub const  SPI_FLASH_SR3_BIT_LC2    : u32 = 2;  /* Latency control bits */
pub const  SPI_FLASH_SR3_BIT_LC1    : u32 = 1;  /* Latency control bits */
pub const  SPI_FLASH_SR3_BIT_LC0    : u32 = 0;  /* Latency control bits */
pub const  SPI_FLASH_SR3_BIT_LC     : u32 = 0;  /* Latency control bits */
pub const  SPI_FLASH_SR3_LC_MASK    : u32 = 0xf<<SPI_FLASH_SR3_BIT_LC;  /* Latency control mask */

pub const  SPI_FLASH_BURST_WRAP_8B  : u32 =  0<<SPI_FLASH_SR3_BIT_BWLEN;
pub const  SPI_FLASH_BURST_WRAP_16B : u32 =  1<<SPI_FLASH_SR3_BIT_BWLEN;
pub const  SPI_FLASH_BURST_WRAP_32B : u32 =  2<<SPI_FLASH_SR3_BIT_BWLEN;
pub const  SPI_FLASH_BURST_WRAP_64B : u32 =  3<<SPI_FLASH_SR3_BIT_BWLEN;

pub const  FLASH_CMD_READ4B         : u8 = 0x13;
pub const  FLASH_CMD_FAST_READ4B    : u8 = 0x0C;
pub const  FLASH_CMD_BLOCK_ERASE4B  : u8 = 0xDC;
pub const  FLASH_CMD_SECTOR_ERASE4B : u8 = 0x21;
pub const  FLASH_CMD_PAGE_PROGRAM4B : u8 = 0x12;

pub struct SPIFlashParam {
    pub capacity : u32,
    pub block_size : u32,
    pub sector_size : u32,
    pub page_size : u32,
    pub device_id : u32,
    pub manufacturer_id : u8,
    pub devicetype_id : u8,
    pub capacity_id : u8,
    pub sfdp : &'static [u8],
}

pub struct SPIFlash{
    selected : bool,
    error : bool,
    sr : [u8;3],
    flash_buf : [u8;FLASH_BUF_SIZE],
    mem : Box<[u8]>,
    flash_cnt : u32,
    addr : u32,
    param : &'static SPIFlashParam,
}

impl SPIWorker for SPIFlash {
    fn init(&mut self)       -> bool{ return true; }
    fn remove(&mut self)     -> bool{ remove(self); return true; }
    fn select(&mut self)     -> bool{ select( self); return true; }
    fn deselect(&mut self)   -> bool{ deselect(self); return true; }
    fn write(&mut self, d:u8)-> u8  { return write(self, d as u32);    }
}

/*
 * Device specific data
 */

pub const SPI_FLASH_PARAM_S25FL164K : SPIFlashParam = SPIFlashParam {
    capacity       : 8*1024*1024,
    block_size     : 64*1024,
    sector_size    :  4*1024,
    page_size      :  1,
    device_id      : 0x15,
    manufacturer_id: 0x01,
    devicetype_id  : 0x40,
    capacity_id    : 0x17,
    sfdp: &[
    // SFDP header 1
    0x53, 0x46, 0x44, 0x50,
    // SFDP header 2
    0x00, 0x01, 0x02, 0xff,
    // Parameter header 1
    0x00, 0x00, 0x01, 0x09,
    // Parameter header 2
    0x80, 0x00, 0x00, 0xFF,
    // Parameter header 3
    0xEF, 0x00, 0x01, 0x04,
    // Parameter header 4
    0xA4, 0x00, 0x00, 0xFF, /* it could be wrong */
    // Parameter header 5
    0x01, 0x00, 0x01, 0x00,
    // Parameter header 6
    0xA4, 0x00, 0x00, 0xFF,

    // 0x30 -
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    // 0x40 -
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    // 0x50 -
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    // 0x60 -
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    // 0x70 -
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,

    // JEDEC Flash Parameters 1
    0xE5, 0x20, 0xF1, 0xFF,
    // JEDEC Flash Parameters 2
    0xFF, 0xFF, 0xFF, 0x02,
    // JEDEC Flash Parameters 3
    0x44, 0xEB, 0x08, 0x6B,
    // JEDEC Flash Parameters 4
    0x08, 0x3B, 0x80, 0xBB,
    // JEDEC Flash Parameters 5
    0xEE, 0xFF, 0xFF, 0xFF,
    // JEDEC Flash Parameters 6
    0xFF, 0xFF, 0xFF, 0xFF,
    // JEDEC Flash Parameters 7
    0xFF, 0xFF, 0xFF, 0xFF,
    // JEDEC Flash Parameters 8
    0x0C, 0x20, 0x00, 0xFF,
    // JEDEC Flash Parameters 9
    0x00, 0xFF, 0x00, 0xFF,
    ],
};

pub const SPI_FLASH_PARAM_MX66U2G45G : SPIFlashParam = SPIFlashParam {
    capacity       : 256*1024*1024,
    block_size     :  64*1024,
    sector_size    :   4*1024,
    page_size      : 1,
    device_id      : 0x3c,
    manufacturer_id: 0xc2,
    devicetype_id  : 0x25,
    capacity_id    : 0x3c,
    sfdp : &[
    // 0x00-
    0x53, 0x46, 0x44, 0x50, 0x06, 0x01, 0x02, 0xff, 0x00, 0x06, 0x01, 0x10, 0x30, 0x00, 0x00, 0xFF,
    // 0x10-
    0xC2, 0x00, 0x01, 0x04, 0x10, 0x01, 0x00, 0xFF, 0x84, 0x00, 0x01, 0x02, 0xC0, 0x00, 0x00, 0xFF,
    // 0x20 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0x30 -
    0xE5, 0x20, 0xFB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x44, 0xEB, 0x08, 0x6B, 0x08, 0x3B, 0x04, 0xBB,
    // 0x40 -
    0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x44, 0xEB, 0x0c, 0x20, 0x0f, 0x52,
    // 0x50 -
    0x10, 0xD8, 0x00, 0xFF, 0x87, 0x49, 0xB5, 0x00, 0x84, 0xD2, 0x04, 0xE2, 0x44, 0x03, 0x67, 0x38,
    // 0x60 -
    0x30, 0xB0, 0x30, 0xB0, 0xF7, 0xBD, 0xD5, 0x5C, 0x4A, 0x9E, 0x29, 0xFF, 0xF0, 0x50, 0xF9, 0x85,
    // 0x70 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0x80 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0x90 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0xA0 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0xB0 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0xC0 -
    0x7F, 0x8F, 0xFF, 0xFF, 0x21, 0x5C, 0xDC, 0xFF, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0xD0 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0xE0 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0xF0 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0x100 -
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // 0x110 -
    0x00, 0x20, 0x50, 0x16, 0x9D, 0xF9, 0xC0, 0x64, 0x85, 0xCB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
]};




pub fn generate_flash(param: &'static SPIFlashParam, data: Box<[u8]>) -> SPIFlash{
    return SPIFlash { 
        selected: false, 
        error: false, 
        sr: [0,0,0], 
        flash_buf: [0;FLASH_BUF_SIZE], 
        mem: data, 
        flash_cnt: 0, 
        addr: 0, 
        param: param,
    };
}

fn remove(flash: &mut SPIFlash){ 
//    flash.mem = ...
    flash.selected = false;
}

fn select(flash: &mut SPIFlash){ 

    if ! flash.selected {
        flash.flash_cnt = 0;
        flash.flash_buf[0] = 0;
        flash.error = false;
        flash.selected = true;
    }
}

fn deselect(flash: &mut SPIFlash){
    if flash.error {
        info!("FLASH deselect (len {}, cmd 0x{:x}, addr 0x{:x})\r", flash.flash_cnt, flash.flash_buf[0], flash.addr);
    }
    flash.flash_cnt = 0;
    flash.flash_buf[0] = 0;
    flash.error = false;
    flash.selected = false;
}

fn write(flash: &mut SPIFlash, val : u32) -> u8 {
    let mut result:u8 = 0xff;
    let count:usize = flash.flash_cnt as usize;

    flash.flash_cnt += 1;

    if count < flash.flash_buf.len() {
        flash.flash_buf[ count ] = val as u8;
    }

    match flash.flash_buf[0] {
        FLASH_CMD_ID => {
            match count {
                0  => {  }
                1  => { return flash.param.manufacturer_id; }
                2  => { return flash.param.devicetype_id; }
                3  => { return flash.param.capacity_id; }
                _  => {  }
            }
        }
        FLASH_CMD_READ => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<16;  }
                2 => { flash.addr|= val<< 8;  }
                3 => { flash.addr|= val;      }
                _ => {
                    result = 0xff;
                    if flash.addr < flash.mem.len() as u32 {
                        result = flash.mem[ flash.addr as usize ];
                    }
                    flash.addr+=1;
                }
            }
        }
        FLASH_CMD_FAST_READ | FLASH_CMD_FAST_READ_DUAL | FLASH_CMD_FAST_READ_QUAD => {
            match  count {
                0 => {  }
                1 => { flash.addr = val<<16;  }
                2 => { flash.addr|= val<< 8;  }
                3 => { flash.addr|= val;      }
                4 => {  }
                _ => {
                    result = 0xff;
                    if flash.addr < flash.mem.len() as u32 {
                        result = flash.mem[ flash.addr as usize ];
                    }
                    flash.addr+=1;
                }
            }
        }
        FLASH_CMD_READ4B => {
            match  count {
                0 => {  }
                1 => { flash.addr = val<<24;  }
                2 => { flash.addr|= val<<16;  }
                3 => { flash.addr|= val<< 8;  }
                4 => { flash.addr|= val;      }
                _ => {
                    result = 0xff;
                    if flash.addr < flash.mem.len() as u32 {
                        result = flash.mem[ flash.addr as usize ];
                    }
                    flash.addr+=1;
                }
            }
        }
        FLASH_CMD_FAST_READ4B => {
            match  count {
                0 => {  }
                1 => { flash.addr = val<<24;  }
                2 => { flash.addr|= val<<16;  }
                3 => { flash.addr|= val<< 8;  }
                4 => { flash.addr|= val;      }
                5 => {  }
                _ => {
                    result = 0xff;
                    if flash.addr < flash.mem.len()  as u32 {
                        result = flash.mem[ flash.addr as usize ];
                    }
                    flash.addr+=1;
                }
            }
        }
        FLASH_CMD_READ_SFDP => {
            match count {
                0 => {  }
                1 => {  }
                2 => {  }
                3 => { flash.addr = val;  }
                4 => {  }
                _ => {
                    result = 0xff;
                    if flash.addr < flash.param.sfdp.len()  as u32  {
                        result = flash.param.sfdp[ flash.addr as usize ];
                    }
                    flash.addr+=1;
                    flash.error = true;
                }
            }
        }
        FLASH_CMD_BLOCK_ERASE => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<16;  }
                2 => { flash.addr|= val<< 8;  }
                3 => { flash.addr|= val;
                    result = 0xff;
                    if 0!=(flash.sr[0] & (1<<SPI_FLASH_SR1_BIT_WE)) {
                        flash.addr &= !(flash.param.block_size - 1);
                        for i in 0..flash.param.block_size {
                            flash.mem[ (flash.addr + i) as usize % flash.mem.len() ] = 0xff;
                        }
                    }
                }
                _ => {  }
            }
        }
        FLASH_CMD_SECTOR_ERASE => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<16;  }
                2 => { flash.addr|= val<< 8;  }
                3 => { flash.addr|= val;
                    result = 0xff;
                    if 0!=(flash.sr[0] & (1<<SPI_FLASH_SR1_BIT_WE)) {
                        flash.addr &= !(flash.param.sector_size - 1);
                        for i in 0..flash.param.sector_size {
                            flash.mem[ (flash.addr + i) as usize % flash.mem.len() ] = 0xff;
                        }
                    }
                }
                _ => {  }
            }
        }
        FLASH_CMD_PAGE_PROGRAM => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<16;  }
                2 => { flash.addr|= val<< 8;  }
                3 => { flash.addr|= val;      }
                _ => {
                    result = 0xff;
                    if 0!=(flash.sr[0] & (1<<SPI_FLASH_SR1_BIT_WE)) {
                        flash.mem[ flash.addr as usize % flash.mem.len() ] = val as u8;
                    }
                    flash.addr+=1;
                }
            }
        }
        FLASH_CMD_BLOCK_ERASE4B => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<24;  }
                2 => { flash.addr|= val<<16;  }
                3 => { flash.addr|= val<< 8;  }
                4 => { flash.addr|= val;
                    result = 0xff;
                    if 0!=(flash.sr[0] & (1<<SPI_FLASH_SR1_BIT_WE)) {
                        flash.addr &= !(flash.param.block_size - 1);
                        for i in 0..flash.param.block_size {
                            flash.mem[ (flash.addr + i) as usize % flash.mem.len() ] = 0xff;
                        }
                    }
                }
                _ => {  }
            }
        }
        FLASH_CMD_SECTOR_ERASE4B => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<24;  }
                2 => { flash.addr|= val<<16;  }
                3 => { flash.addr|= val<< 8;  }
                4 => { flash.addr|= val;
                    result = 0xff;
                    if 0!=(flash.sr[0] & (1<<SPI_FLASH_SR1_BIT_WE)) {
                        flash.addr &= !(flash.param.sector_size - 1);
                        for i in 0..flash.param.sector_size {
                            flash.mem[ (flash.addr + i) as usize % flash.mem.len() ] = 0xff;
                        }
                    }
                }
                _ => {  }
            }
        }
        FLASH_CMD_PAGE_PROGRAM4B => {
            match count {
                0 => {  }
                1 => { flash.addr = val<<24;  }
                2 => { flash.addr|= val<<16;  }
                3 => { flash.addr|= val<< 8;  }
                4 => { flash.addr|= val;      }
                _ => {
                    result = 0xff;
                    if 0!=(flash.sr[0] & (1<<SPI_FLASH_SR1_BIT_WE)) {
                        flash.mem[ flash.addr as usize % flash.mem.len() ] = val as u8;
                    }
                    flash.addr+=1;
                }
            }
        }
        FLASH_CMD_READ_SR1 => {
            match count {
                0 => {  }
                1 => { return flash.sr[0] & (!(1<<SPI_FLASH_SR1_BIT_BUSY)); }
                _ => {  }
            }
        }
        FLASH_CMD_WRITE_ENABLE => {
            flash.sr[0] |=  1<< SPI_FLASH_SR1_BIT_WE;
        }
        FLASH_CMD_WRITE_DISABLE => {
            flash.sr[0] &= !(1<< SPI_FLASH_SR1_BIT_WE);
        }
        _ => {
            if ! flash.error { 
                info!("SPI command 0x{:x} is issued. But it cannot be handled\r", flash.flash_buf[0]);
            }
            flash.error = true;
        }
    }

    return result;
}

