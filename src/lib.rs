#![allow(dead_code)]

mod procstate;
mod mips;
mod config;
mod cp0def;
mod cp0;
mod mem;
mod exec_common;
mod exec_mips16;
mod exec_mips32;
mod exception;
mod tlb;
mod addr_cache;
mod dev_uart;
mod dev_soc;
mod dev_spi;
mod dev_spiflash;
mod mainloop;

// native app. only
mod time_trig;
#[cfg(not(target_family = "wasm"))] mod stin;

// wasm only 
mod utils;
mod wasm_utils;
use wasm_bindgen::prelude::*;


pub use crate::dev_spiflash::SPIFlashParam;

/*
#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-term!");
}
*/

#[wasm_bindgen]
pub async fn mips_emu(){

    loop{
        wasm_utils::clear_image_data();
        wasm_utils::print_string(format!("Please upload a system image file...\r\n"));

        let flash_param : &SPIFlashParam;
        let mut flashdata;
    
        loop{
            wasm_utils::print_string(format!("."));
            wasm_utils::sleep(1000).await;
            if wasm_utils::check_image_avail().as_bool().unwrap() {
                match wasm_utils::get_requested_flash_capacity() {
                    256 => { flash_param =  &dev_spiflash::SPI_FLASH_PARAM_MX66U2G45G; }
                    _   => { flash_param =  &dev_spiflash::SPI_FLASH_PARAM_S25FL164K;  }
                }
                flashdata = vec![0xff as u8; flash_param.capacity as usize].into_boxed_slice();

                let d = wasm_utils::get_image_data();
                let min =  Ord::min(d.byte_length() as usize, flashdata.len());
                for i in 0..min {
                    flashdata[i] = d.get_index(i as u32);
                }
                break;
            }
        }
        let mut ms = exrmips::generate_machine_state(flash_param,  &flashdata);

        wasm_utils::print_string(format!("\r\nStarting the emulator...\r\n"));
        exrmips::run_wasm(&mut ms).await;
        wasm_utils::print_string(format!("\r\nEmulator terminating..."));
    }
}

pub mod exrmips{
    pub use crate::dev_spiflash::SPI_FLASH_PARAM_S25FL164K;
    pub use crate::dev_spiflash::SPI_FLASH_PARAM_MX66U2G45G;

    use crate::dev_uart;
    use crate::procstate::{EmuSetting, Reg, MachineState};
    use crate::mem::MemRegion;
    use crate::tlb::TLBEntry;
    use crate::dev_uart::IoUART;
    use crate::dev_soc::{IoGPIO, IoMisc};
    use crate::dev_spiflash::{SPIFlash, SPIFlashParam};
    use crate::dev_spi::IoSPI;

    use crate::{cp0def, config, mips, dev_spiflash, mainloop};
    use crate::time_trig;
    use crate::c0_val;

    #[cfg(not(target_family = "wasm"))]
    use crate::stin;

    pub async fn run_wasm(ms: &mut MachineState) { mainloop::run_wasm(ms).await; }
    
    #[cfg(not(target_family = "wasm"))]
    pub fn run_term(ms: &mut MachineState) { mainloop::run_term(ms); }

    
    pub fn generate_machine_state(flash_param: &'static SPIFlashParam, bindata: &[u8]) -> MachineState {

        #[cfg(not(target_family = "wasm"))]
        let stin_obj = stin::spawn_stdin_channel();

        let mut ms = MachineState { 
            reg: Reg::new(),
            mem: MemRegion::new(),
            tlb: [ TLBEntry::new(); config::NUM_TLB_ENTRY as usize ],
            tlbcache: [ config::NUM_TLB_ENTRY as u8; config::TLB_CACHE_SIZE ],
            emu: EmuSetting { breakpoint:0, breakmask:0xffffffff, runafterbreak:0, breakcounter:0, nexec_insts:0, execrate:0, stopcount:0, debug:false },
            uart: IoUART::new(), 
            gpio: IoGPIO::new(),
            spi: IoSPI::new(),
            misc: IoMisc::new(),
            sleep_req: false,

            #[cfg(not(target_family = "wasm"))]
            stdin_ch: Box::new(dev_uart::NativeUARTConsole{receiver: stin_obj.0}),
            #[cfg(target_family = "wasm")]
            stdin_ch: Box::new(dev_uart::WasmUARTConsole{}),
            #[cfg(not(target_family = "wasm"))]
            ctrlc_count: stin_obj.1,
            #[cfg(not(target_family = "wasm"))]
            time_trigger: time_trig::spawn_time_trigger(),
        };

        // prepares memory region of flash memory size and copies the image into the region
        let mut flashdata = vec![0xff as u8; flash_param.capacity as usize].into_boxed_slice();
        flashdata[0..bindata.len()].copy_from_slice(bindata);

        // generate SPIFlash 
        let spiflash:SPIFlash = dev_spiflash::generate_flash(flash_param, flashdata );

        // registers the SPIFlash as SPI0
        ms.spi.workers[0] = Box::new( spiflash );

        // sets the initial PC value
        ms.reg.pc = mips::EXCEPT_VECT_RESET;

        // Initializing register values of CoProcessor 0
        c0_val!( ms.reg, cp0def::C0_STATUS ) = cp0def::C0_STATUS_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_CONFIG ) = cp0def::C0_CONFIG_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_CONFIG1) = cp0def::C0_CONFIG1_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_CONFIG2) = cp0def::C0_CONFIG2_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_EBASE)   = cp0def::C0_EBASE_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_PRID)    = cp0def::C0_PRID_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_ENTRYHI) = cp0def::C0_ENTRYHI_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_ENTRYLO0)= cp0def::C0_ENTRYLO0_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_ENTRYLO1)= cp0def::C0_ENTRYLO1_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_RANDOM)  = cp0def::C0_RANDOM_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_INDEX)   = cp0def::C0_INDEX_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_PAGEMASK)= cp0def::C0_PAGEMASK_SETTING.init_val;
        c0_val!( ms.reg, cp0def::C0_WIRED)   = cp0def::C0_WIRED_SETTING.init_val;
        ms
    }
}