use crate::addr_cache::AddrCache;
use crate::config;
use crate::dev_uart;
use crate::mips;
use crate::cp0def;
use crate::mem::MemRegion;
use crate::tlb::TLBEntry;
use crate::dev_uart::IoUART;
use crate::dev_soc::IoGPIO;
use crate::dev_soc::IoMisc;
use crate::dev_spi::IoSPI;

use std::sync::Arc;
use std::sync::atomic;

use crate::c0_val;

use log::info;



impl Reg {
    pub fn new() -> Self {
        Self { 
            r  : [0; 32], 
            pc : 0,
            pc_delay : 0,
            pc_prev_jump : 0,
            hi : 0,
            lo : 0,
        
            delay_en : false,
            ll_sc : false,
        
            c0_count_basetime : 0,       /* base time in usec */
            c0_count_currenttime : 0,    /* current time in usec */
            c0_count_ninst_in_ctime : 0, /* nExecInsts in the current time */
            c0_compare_long : 0,         /* long version of c0_compare */
            pc_cache: AddrCache::new(),
            dr_cache: [AddrCache::new(), AddrCache::new()],
            dw_cache: [AddrCache::new(), AddrCache::new()],
            cp0 : [0; 1<<(mips::CP_REG_BITS + mips::CP_SEL_BITS)],
        }
    }
}

pub struct Reg {
    pub r : [u32; 32],
    pub pc : u32,

    pub pc_delay : u32,
    pub pc_prev_jump : u32,
    pub hi : u32,
    pub lo : u32,

    pub delay_en : bool,
    pub ll_sc : bool,

    pub c0_count_basetime : u64,       /* base time in usec */
    pub c0_count_currenttime : u64,    /* current time in usec */
    pub c0_count_ninst_in_ctime : u64, /* nExecInsts in the current time */
    pub c0_compare_long : u64,         /* long version of c0_compare */

    pub cp0 : [u32; 1<<(mips::CP_REG_BITS + mips::CP_SEL_BITS)],

    pub pc_cache: AddrCache,
    pub dr_cache: [AddrCache; 2],
    pub dw_cache: [AddrCache; 2],
}

pub struct EmuSetting{
    pub breakpoint : u32,
    pub breakmask  : u32,
    pub runafterbreak : u64,
    pub breakcounter  : u64,
    pub nexec_insts   : u64,
	pub execrate      : u64, /* executed instructions per second in the last host timer period */
    pub stopcount : u64,
    pub debug : bool,
}

pub struct MachineState {
    pub reg : Reg,
    pub mem : MemRegion,
    pub tlb : [TLBEntry; config::NUM_TLB_ENTRY as usize],
    pub tlbcache: [u8; config::TLB_CACHE_SIZE],
    pub uart: IoUART,
    pub misc: IoMisc,
    pub gpio: IoGPIO,
    pub spi : IoSPI,
    pub emu : EmuSetting,
    pub sleep_req : bool,
    pub stdin_ch  : Box<dyn dev_uart::UartReadWrite>,
    #[cfg(not(target_family = "wasm"))]
    pub ctrlc_count : Arc<atomic::AtomicUsize>,
    #[cfg(not(target_family = "wasm"))]
    pub time_trigger: Arc<atomic::AtomicBool>,
}

pub fn log_print_reg32(reg: &Reg){
    info!("PC = {:>08x} C0_STATUS = {:x}\r", reg.pc, c0_val!(reg, cp0def::C0_STATUS) );
    info!("r[ 0.. 7]={:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x}\r", reg.r[ 0], reg.r[ 1], reg.r[ 2], reg.r[ 3], reg.r[ 4], reg.r[ 5], reg.r[ 6], reg.r[ 7]);
    info!("r[ 8..15]={:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x}\r", reg.r[ 8], reg.r[ 9], reg.r[10], reg.r[11], reg.r[12], reg.r[13], reg.r[14], reg.r[15]);
    info!("r[16..23]={:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x}\r", reg.r[16], reg.r[17], reg.r[18], reg.r[19], reg.r[20], reg.r[21], reg.r[22], reg.r[23]);
    info!("r[24..31]={:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x} {:>08x}\r", reg.r[24], reg.r[25], reg.r[26], reg.r[27], reg.r[28], reg.r[29], reg.r[30], reg.r[31]);
}

pub fn dump_mem(mem : &Box<[u8]>, start: u32, len: u32){

    for i in (start>>4)..(start+len)>>4 {
        print!("{:>08x}:", i<<4);
        for j in 0..16{
            print!(" {:>02x}", mem[ ((i<<4) + j) as usize]);
        }
        println!("");
    }
}

