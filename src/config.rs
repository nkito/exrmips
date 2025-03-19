
#![allow(dead_code)]

pub const FREQ_CPU                     : u32 = 400*1000*1000;
pub const CPU_FREQ_COUNT_RESOLUTION    : u32 = 2;  /* CCRes value of RDHWR 3 */
pub const SYSTEM_TIMER_INTERVAL_IN_USEC: u32 = 1000;

pub const NUM_TLB_ENTRY: u32 = 32;
pub const TLB_CACHE_BITS : usize = 10;
pub const TLB_CACHE_SIZE : usize = 1<<TLB_CACHE_BITS;

// RAM area is at most 256MB
pub const RAM_AREA_ADDR : u32 = 0x00000000;
pub const RAM_AREA_SIZE : u32 = 0x10000000;

// main memory size (2^26 = 64MB)
pub const DRAM_ADDR_WIDTH : u32   = 26; // up to 28 (256MB)
pub const DRAM_SIZE       : usize = 1<<DRAM_ADDR_WIDTH;
pub const DRAM_ADDR_MASK  : u32   = DRAM_SIZE as u32 - 1;

// ROM area is at most 16MB
pub const ROM_AREA_ADDR : u32 = 0x1f000000;
pub const ROM_AREA_SIZE : u32 = 0x01000000;
