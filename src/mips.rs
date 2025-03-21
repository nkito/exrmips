#![allow(dead_code)]

pub const CP_REG_BITS: i32 = 5;
pub const CP_SEL_BITS: i32 = 4;


pub const KUSEG       : u32 = 0x00000000;
pub const KUSEG_SIZE  : u32 = 0x80000000;
pub const KSEG0       : u32 = 0x80000000;
pub const KSEG0_SIZE  : u32 = 0x20000000;
pub const KSEG1       : u32 = 0xA0000000;
pub const KSEG1_SIZE  : u32 = 0x20000000;
pub const KSEG2       : u32 = 0xC0000000;
pub const KSEG2_SIZE  : u32 = 0x20000000;
pub const KSEG3       : u32 = 0xE0000000;
pub const KSEG3_SIZE  : u32 = 0x20000000;

pub const EXCEPT_VECT_RESET : u32 = 0xbfc00000;

#[macro_export]
macro_rules! except_vect_cache_err { ($ebase:expr, $bev:expr) => 
(if 0!=$bev {                           0xbfc00300 }else{                            0xA0000100  |($ebase&0x1ffff000)}) }
#[macro_export]
macro_rules! except_vect_tlb_refill{ ($ebase:expr, $bev:expr, $exl:expr) => 
(if 0!=$bev {if $exl   {0xbfc00380}else{0xbfc00200}}else{(if $exl   {0x80000180}else{0x80000000})|($ebase&0x3ffff000)}) }
#[macro_export]
macro_rules! except_vect_int       { ($ebase:expr, $bev:expr, $iv:expr ) => 
(if 0!=$bev {if 0!=$iv {0xbfc00400}else{0xbfc00380}}else{(if 0!=$iv {0x80000200}else{0x80000180})|($ebase&0x3ffff000)}) }
#[macro_export]
macro_rules! except_vect_all_other { ($ebase:expr, $bev:expr) => 
(if 0!=$bev {                           0xbfc00380 }else{                            0x80000180  |($ebase&0x3ffff000)}) }



#[macro_export]
macro_rules! kseg01_to_paddr { ( $i:expr ) => ($i & 0x1fffffff) }


pub const REGSTR : [&str; 32] = [
    //   0      1      2      3      4      5      6      7
    "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3",
    //   8      9     10     11     12     13     14     15
      "$t0", "$t1", "$t2", "$t3", "$t4", "$t5", "$t6", "$t7",
    //  16     17     18     19     20     21     22     23
      "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7",
    //  24     25     26     27     28     29     30     31
      "$t8", "$t9", "$k0", "$k1", "$gp", "$sp", "$fp", "$ra"
];
