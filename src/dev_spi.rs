use log::info;

// Base addresses
pub const SPI0_BASE_ADDRESS:u32 = 0x1f000000;

pub const SPI_ADDR_SIZE :u32 = 0x1c;

pub const SPI_ADDR_MASK :u32 = 0x1f;

pub const SPI_MAX_SLAVES :u32 =  3;

/*
 * SPI serial flash registers
 */
pub const SPI_FUNC_SEL_REG      :u32 = 0x00;
pub const SPI_CTRL_REG          :u32 = 0x04;
pub const SPI_IO_CTRL_REG       :u32 = 0x08;
pub const SPI_READ_DATA_REG     :u32 = 0x0c;
pub const SPI_SHIFT_DATAOUT_REG :u32 = 0x10;
pub const SPI_SHIFT_CNT_REG     :u32 = 0x14;
pub const SPI_SHIFT_DATAIN_REG  :u32 = 0x18;

pub const SPI_CTRL_BIT_REMAP_DISABLE :u32 = 6;

pub const SPI_IO_CTRL_BIT_IO_DO   :u32 =  0;
pub const SPI_IO_CTRL_BIT_IO_CLK  :u32 =  8;
pub const SPI_IO_CTRL_BIT_IO_CS0  :u32 = 16;
pub const SPI_IO_CTRL_BIT_IO_CS1  :u32 = 17;
pub const SPI_IO_CTRL_BIT_IO_CS2  :u32 = 18;

pub const SPI_SHIFT_CNT_BIT_TERMINATE       :u32 = 26;
pub const SPI_SHIFT_CNT_BIT_SHIFT_CLKOUT    :u32 = 27;
pub const SPI_SHIFT_CNT_BIT_SHIFT_CHNL_CS0  :u32 = 28;
pub const SPI_SHIFT_CNT_BIT_SHIFT_CHNL_CS1  :u32 = 29;
pub const SPI_SHIFT_CNT_BIT_SHIFT_CHNL_CS2  :u32 = 30;
pub const SPI_SHIFT_CNT_BIT_SHIFT_EN        :u32 = 31;
pub const SPI_SHIFT_CNT_BIT_SHIFT_COUNT     :u32 = 0;
pub const SPI_SHIFT_CNT_SHIFT_COUNT_MASK    :u32 = 0x7f;

pub trait SPIWorker {
    fn init(&mut self)       -> bool;
    fn remove(&mut self)     -> bool;
    fn select(&mut self)     -> bool;
    fn deselect(&mut self)   -> bool;
    fn write(&mut self, d:u8)-> u8;
}

pub struct IoSPI{
    pub function_select : u32,
    pub control : u32,
    pub io_control : u32,
    pub read_data_addr : u32,
    pub shift_dataout : u32,
    pub shift_count : u32,
    pub shift_datain : u32,
    pub workers : [Box<dyn SPIWorker>;SPI_MAX_SLAVES as usize],
}

pub struct DummySPIDevice { }

impl SPIWorker for DummySPIDevice {
    fn init(&mut self)       -> bool{ return true; }
    fn remove(&mut self)     -> bool{ return true; }
    fn select(&mut self)     -> bool{ return true; }
    fn deselect(&mut self)   -> bool{ return true; }
    fn write(&mut self,_d:u8)-> u8  { return 0;    }
}


impl IoSPI {
    pub fn new() -> Self {
        Self { 
            function_select: 0, 
            control: 0,
            io_control: 0,
            read_data_addr: 0,
            shift_dataout: 0,
            shift_count: 0,
            shift_datain: 0,
            workers: [Box::new(DummySPIDevice{}), Box::new(DummySPIDevice{}), Box::new(DummySPIDevice{})],
        }
    }
}


pub fn init(spi: &mut IoSPI){
    for i in 0..spi.workers.len(){
        spi.workers[i].init();
    }
}

pub fn remove(spi: &mut IoSPI){
    for i in 0..spi.workers.len(){
        spi.workers[i].remove();
    }
}

pub fn read_reg(spi: &IoSPI, addr:u32) -> u32{

    return match (addr&SPI_ADDR_MASK) & (!0x03) {
        SPI_FUNC_SEL_REG     => spi.function_select,
        SPI_CTRL_REG         => spi.control,
        SPI_IO_CTRL_REG      => spi.io_control,
        SPI_READ_DATA_REG    => spi.read_data_addr,
        SPI_SHIFT_DATAOUT_REG=> spi.shift_dataout,
        SPI_SHIFT_CNT_REG    => spi.shift_count,
        SPI_SHIFT_DATAIN_REG => spi.shift_datain,
        _ =>  0,
    }
}

pub fn write_reg(spi: &mut IoSPI, addr:u32, data:u32){

    let cnl_mask : [u32;SPI_MAX_SLAVES as usize] = [
        (1<<SPI_SHIFT_CNT_BIT_SHIFT_CHNL_CS0), 
        (1<<SPI_SHIFT_CNT_BIT_SHIFT_CHNL_CS1), 
        (1<<SPI_SHIFT_CNT_BIT_SHIFT_CHNL_CS2)
    ];

    match (addr&SPI_ADDR_MASK) & (!0x03) {
        SPI_FUNC_SEL_REG => {
            spi.function_select = data;
        }
        SPI_CTRL_REG => {
            spi.control = data;
        }
        SPI_IO_CTRL_REG => {
            spi.io_control = data;
        }
        SPI_READ_DATA_REG => {
            spi.read_data_addr = data;
        }
        SPI_SHIFT_DATAOUT_REG =>{
            spi.shift_dataout = data;
        }
        SPI_SHIFT_CNT_REG => {
            /*
             * Changing chip-select bits without SHIFT_EN bit and shift-count seems to change no chip select outputs.
             */

            let mut d:u32 = data;

            for i in 0..SPI_MAX_SLAVES as usize {
                if 0!=((d ^ spi.shift_count) & cnl_mask[i]) && 0!=(d & (1<<SPI_SHIFT_CNT_BIT_SHIFT_EN)) && (d & SPI_SHIFT_CNT_SHIFT_COUNT_MASK) > 0 {
                    if 0!=(d & cnl_mask[i]) {
                        spi.workers[i].select();
                    }else{
                        spi.workers[i].deselect();
                    }
                }
            }

            if 0 != (d & (1<<SPI_SHIFT_CNT_BIT_SHIFT_EN)) {

                // Checking for each chip select
                for i in 0..SPI_MAX_SLAVES as usize {

                    // tx is carried out for enabled chip selects
                    if 0 != (d & cnl_mask[i]) {
                        if (d & SPI_SHIFT_CNT_SHIFT_COUNT_MASK) > 0 {
                            let cnt:u32 = (d & SPI_SHIFT_CNT_SHIFT_COUNT_MASK)/8;
                            let outdata = spi.shift_dataout;
                            let mut indata:u32  = 0;
                            for j in (0..cnt).rev() {
                                indata = (indata << 8) | (spi.workers[i].write( ((outdata>>(8*j))&0xff) as u8 ) as u32);
                            }
                            spi.shift_datain = indata;
                        }
                        if 0 != (d & (1<<SPI_SHIFT_CNT_BIT_TERMINATE)) {
                            spi.workers[i].deselect();
                            d &= !cnl_mask[i];
                        }
                    }
                }
                d &= !(1<<SPI_SHIFT_CNT_BIT_SHIFT_EN);
            }

            spi.shift_count = d;
        }
        SPI_SHIFT_DATAIN_REG => {
            spi.shift_datain = data;
        }
        _ => {
            info!("unknown SPI address was written (addr 0x{:x} val 0x{:x})\n", addr, data);
        }
    }
}

