use crate::procstate::MachineState;
use crate::exception;
use crate::mips;
use crate::mem;

//use crate::exec_common;
use crate::sign_ext16;
use crate::sign_ext15;
use crate::zero_ext16;
use crate::sign_ext11;
use crate::sign_ext8;
use crate::zero_ext8;
use crate::sign_ext4;
use crate::update_pc_next32_with_delayed_imm;
use crate::update_pc_next16_with_delayed_imm;
use crate::update_pc_next32;
use crate::update_pc_next16;
use crate::update_pc_imm;

use log::{error,info};


pub const MIPS16E_OP_ADDIUSP : u32 = 0x00;
pub const MIPS16E_OP_ADDIUPC : u32 = 0x01;
pub const MIPS16E_OP_B       : u32 = 0x02;
pub const MIPS16E_OP_JAL     : u32 = 0x03;
pub const MIPS16E_OP_BEQZ    : u32 = 0x04;
pub const MIPS16E_OP_BNEZ    : u32 = 0x05;
pub const MIPS16E_OP_SHIFT   : u32 = 0x06;
pub const MIPS16E_OP_RRIA    : u32 = 0x08;
pub const MIPS16E_OP_ADDIU8  : u32 = 0x09;
pub const MIPS16E_OP_SLTI    : u32 = 0x0A;
pub const MIPS16E_OP_SLTIU   : u32 = 0x0B;
pub const MIPS16E_OP_I8      : u32 = 0x0C;
pub const MIPS16E_OP_LI      : u32 = 0x0D;
pub const MIPS16E_OP_CMPI    : u32 = 0x0E;
pub const MIPS16E_OP_LB      : u32 = 0x10;
pub const MIPS16E_OP_LH      : u32 = 0x11;
pub const MIPS16E_OP_LWSP    : u32 = 0x12;
pub const MIPS16E_OP_LW      : u32 = 0x13;
pub const MIPS16E_OP_LBU     : u32 = 0x14;
pub const MIPS16E_OP_LHU     : u32 = 0x15;
pub const MIPS16E_OP_LWPC    : u32 = 0x16;
pub const MIPS16E_OP_SB      : u32 = 0x18;
pub const MIPS16E_OP_SH      : u32 = 0x19;
pub const MIPS16E_OP_SWSP    : u32 = 0x1A;
pub const MIPS16E_OP_SW      : u32 = 0x1B;
pub const MIPS16E_OP_RRR     : u32 = 0x1C;
pub const MIPS16E_OP_RR      : u32 = 0x1D;
pub const MIPS16E_OP_EXTEND  : u32 = 0x1E;

pub const MIPS16E_RRFUNCT_JR     : u32 = 0x00;
pub const MIPS16E_RRFUNCT_SDBBP  : u32 = 0x01;
pub const MIPS16E_RRFUNCT_SLT    : u32 = 0x02;
pub const MIPS16E_RRFUNCT_SLTU   : u32 = 0x03;
pub const MIPS16E_RRFUNCT_SLLV   : u32 = 0x04;
pub const MIPS16E_RRFUNCT_BREAK  : u32 = 0x05;
pub const MIPS16E_RRFUNCT_SRLV   : u32 = 0x06;
pub const MIPS16E_RRFUNCT_SRAV   : u32 = 0x07;
pub const MIPS16E_RRFUNCT_CMP    : u32 = 0x0A;
pub const MIPS16E_RRFUNCT_NEG    : u32 = 0x0B;
pub const MIPS16E_RRFUNCT_AND    : u32 = 0x0C;
pub const MIPS16E_RRFUNCT_OR     : u32 = 0x0D;
pub const MIPS16E_RRFUNCT_XOR    : u32 = 0x0E;
pub const MIPS16E_RRFUNCT_NOT    : u32 = 0x0F;
pub const MIPS16E_RRFUNCT_MFHI   : u32 = 0x10;
pub const MIPS16E_RRFUNCT_CNVT   : u32 = 0x11;
pub const MIPS16E_RRFUNCT_MFLO   : u32 = 0x12;
pub const MIPS16E_RRFUNCT_MULT   : u32 = 0x18;
pub const MIPS16E_RRFUNCT_MULTU  : u32 = 0x19;
pub const MIPS16E_RRFUNCT_DIV    : u32 = 0x1A;
pub const MIPS16E_RRFUNCT_DIVU   : u32 = 0x1B;

macro_rules! xlat { ( $i:expr ) => (if $i < 2 { ($i+16) as usize }else{ $i as usize }) }

macro_rules! with_prefix{ ($i:expr) => (0!=(($i) & 0xffff0000)) }


macro_rules! unknown_instruction16{
    ( $inst:expr, $msg:expr ) => 
    {
        error!("Unknown MIPS16 instruction (inst={:>08x}, {})", $inst, $msg);
        return false;
    }
}


pub fn exec(ms: &mut MachineState, inst32 : u32) -> bool {
//    let pointer : u32 = ms.reg.pc;

    let  op5  : u32 = (inst32>>11) & 0x1f; /* inst[15:11] */
    let  rx   : u32 = (inst32>> 8) & 0x07; /* inst[10: 8] */
    let  ry   : u32 = (inst32>> 5) & 0x07; /* inst[ 7: 5] */
    let  rz   : u32 = (inst32>> 2) & 0x07; /* inst[ 4: 2] */
    let  sa3  : u32 = (inst32>> 2) & 0x07; /* inst[ 4: 2] */
    let funct5: u32 =  inst32      & 0x1f; /* inst[ 5: 0] */
    let  imm8 : u32 =  inst32      & 0xff; /* inst[ 7: 0] */

    let imm16 = ((inst32>>5)&(0x1f<<11)) | ((inst32>>16)&(0x3f<<5)) | funct5;
    let imm15 = ((inst32>>5)&(0x0f<<11)) | ((inst32>>16)&(0x7f<<4)) | (inst32&0xf);

    let mut utmp :u32;
    let  utmp2: u32;

    if (inst32>>27) == MIPS16E_OP_JAL {
        let mut jumpaddr :u32;
        if 0!=( inst32 & (1<<26) ){
            jumpaddr = inst32 & 0xffff;           // [16: 0]
            jumpaddr|= ((inst32>>21) & 0x1f)<<16; // [20:16]
            jumpaddr|= ((inst32>>16) & 0x1f)<<21; // [25:21]
            ms.reg.r[31] = ms.reg.pc + 6;
            update_pc_next32_with_delayed_imm!( ms, ((ms.reg.pc+4) & ((!(0x03ffffff as u32))<<2)) + (jumpaddr<<2) );
            if ms.emu.debug { info!("jalx 0x{:x}(={:x})\n", jumpaddr, ms.reg.pc_delay); }
        }else{
            // JAL (which preserves the current ISA)
            jumpaddr = inst32 & 0xffff;           // [16: 0]
            jumpaddr|= ((inst32>>21) & 0x1f)<<16; // [20:16]
            jumpaddr|= ((inst32>>16) & 0x1f)<<21; // [25:21]
            ms.reg.r[31] = ms.reg.pc + 6;
            // +1 is to keep execution in MIPS16e mode
            update_pc_next32_with_delayed_imm!( ms, ((ms.reg.pc+4) & ((!(0x03ffffff as u32))<<2)) + (jumpaddr<<2) +1);
            if ms.emu.debug { info!("jal 0x{:x}(={:x})\n", jumpaddr, ms.reg.pc_delay); }
        }
        return true;
    }

    match op5 {
        MIPS16E_OP_ADDIUSP => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("addiu {}, sp, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[xlat!(rx)] = ms.reg.r[29] + sign_ext16!(imm16);
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("addiu {}, sp, 0x{:x}\n", mips::REGSTR[xlat!(rx)], zero_ext8!(imm8)<<2); }
                ms.reg.r[xlat!(rx)] = ms.reg.r[29] + (zero_ext8!(imm8)<<2);
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_ADDIUPC => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("addiu {}, pc, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[xlat!(rx)] = (ms.reg.pc & !(3 as u32)) + sign_ext16!(imm16);
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("addiu {}, pc, 0x{:x}\n", mips::REGSTR[xlat!(rx)], zero_ext8!(imm8)<<2); }
                utmp = ((if ms.reg.delay_en { ms.reg.pc_prev_jump }else{ ms.reg.pc }) & !(3 as u32)) + (zero_ext8!(imm8)<<2);
                ms.reg.r[xlat!(rx)] = utmp;
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_B => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("b 0x{:x}(={:x})\n", imm16, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4); }
                update_pc_imm!(ms, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4);
            }else{
                utmp = inst32 & 0x7ff;
                if ms.emu.debug { info!("b 0x{:x}(={:x})\n", imm8, ms.reg.pc + (sign_ext11!(utmp) << 1) + 2); }
                update_pc_imm!(ms, ms.reg.pc + (sign_ext11!(utmp) << 1) + 2);
            }
        }
        MIPS16E_OP_JAL => {
            /* not processed here */
            unknown_instruction16!(inst32, "unhandled MIPS16 op (JAL)");
        }
        MIPS16E_OP_BEQZ => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("beqz {}, 0x{:x}(={:x})\n", mips::REGSTR[xlat!(rx)], imm16, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4); }
                if ms.reg.r[xlat!(rx)] == 0 {
                    update_pc_imm!(ms, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4);
                }else{
                    update_pc_next32!(ms);
                }
            }else{
                if ms.emu.debug { info!("beqz {}, 0x{:x}(={:x})\n", mips::REGSTR[xlat!(rx)], imm8, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2); }
                if ms.reg.r[xlat!(rx)] == 0 {
                    update_pc_imm!(ms, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2);
                }else{
                    update_pc_next16!(ms);
                }
            }
        }
        MIPS16E_OP_BNEZ => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("bnez {}, 0x{:x}(={:x})\n", mips::REGSTR[xlat!(rx)], imm16, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4); }
                if ms.reg.r[xlat!(rx)] != 0 {
                    update_pc_imm!(ms, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4);
                }else{
                    update_pc_next32!(ms);
                }
            }else{
                if ms.emu.debug { info!("bnez {}, 0x{:x}(={:x})\n", mips::REGSTR[xlat!(rx)], imm8, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2); }
                if ms.reg.r[xlat!(rx)] != 0 {
                    update_pc_imm!(ms, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2);
                }else{
                    update_pc_next16!(ms);
                }
            }
        }
        MIPS16E_OP_SHIFT => {
            match funct5 & 3 {
                0 => /* SLL */ {
                    if with_prefix!(inst32) {
                        let sa:u32 = (inst32>>22) & 0x1f;
                        if ms.emu.debug { info!("sll {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)], sa); }
                        ms.reg.r[xlat!(rx)] = ms.reg.r[xlat!(ry)] << sa;
                        update_pc_next32!(ms);
                    }else{
                        let sa:u32 = if sa3==0 { 8 }else{ sa3 };
                        if ms.emu.debug { info!("sll {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)], sa); }
                        ms.reg.r[xlat!(rx)] = ms.reg.r[xlat!(ry)] << sa;
                        update_pc_next16!(ms);
                    }
                }
                2 => /* SRL */ {
                    if with_prefix!(inst32) {
                        let sa:u32 = (inst32>>22) & 0x1f;
                        if ms.emu.debug { info!("srl {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)], sa); }
                        ms.reg.r[xlat!(rx)] = (((ms.reg.r[xlat!(ry)] as u32) as u64) >> sa) as u32;
                        update_pc_next32!(ms);
                    }else{
                        let sa:u32 = if sa3==0 { 8 }else{ sa3 };
                        if ms.emu.debug { info!("srl {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)], sa); }
                        ms.reg.r[xlat!(rx)] = (((ms.reg.r[xlat!(ry)] as u32) as u64) >> sa) as u32;
                        update_pc_next16!(ms);
                    }
                }
                3 => /* SRA */ {
                    if with_prefix!(inst32) {
                        let sa:u32 = (inst32>>22) & 0x1f;
                        if ms.emu.debug { info!("sra {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)], sa); }
                        ms.reg.r[xlat!(rx)] = (((ms.reg.r[xlat!(ry)] as i32) as i64) >> sa) as u32;
                        update_pc_next32!(ms);
                    }else{
                        let sa:u32 = if sa3==0 { 8 }else{ sa3 };
                        if ms.emu.debug { info!("sra {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)], sa); }
                        ms.reg.r[xlat!(rx)] = (((ms.reg.r[xlat!(ry)] as i32) as i64) >> sa) as u32;
                        update_pc_next16!(ms);
                    }
                }
                _ => {
                    unknown_instruction16!(inst32, "unhandled MIPS16 op (SHIFT)");
                }
            }
        }
        MIPS16E_OP_RRIA => {
            if 0!=( inst32 & (1<<4) ) {
                unknown_instruction16!(inst32, "unhandled MIPS16 op (RRIA)");
            }else{
                if with_prefix!(inst32) {
                    if ms.emu.debug { info!("addiu {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(ry)], mips::REGSTR[xlat!(rx)], imm15); }
                    ms.reg.r[xlat!(ry)] = ms.reg.r[xlat!(rx)] + sign_ext15!(imm15);
                    update_pc_next32!(ms);
                }else{
                    if ms.emu.debug { info!("addiu {}, {}, 0x{:x}\n", mips::REGSTR[xlat!(ry)], mips::REGSTR[xlat!(rx)], inst32&0xf); }
                    ms.reg.r[xlat!(ry)] = ms.reg.r[xlat!(rx)] + sign_ext4!(inst32 & 0xf);
                    update_pc_next16!(ms);
                }
            }
        }
        MIPS16E_OP_ADDIU8 => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("addiu {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[xlat!(rx)] += sign_ext16!(imm16);
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("addiu {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm8); }
                ms.reg.r[xlat!(rx)] += sign_ext8!(imm8);
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_SLTI => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("slti {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[24] = if (ms.reg.r[xlat!(rx)] as i32) < (sign_ext16!(imm16) as i32) { 1 }else{ 0 };
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("slti {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm8); }
                ms.reg.r[24] = if (ms.reg.r[xlat!(rx)] as i32) < (zero_ext8!(imm8) as i32) { 1 }else{ 0 };
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_SLTIU => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("sltiu {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[24] = if ms.reg.r[xlat!(rx)] < (sign_ext16!(imm16) as u32) { 1 }else{ 0 };
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("sltiu {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm8); }
                ms.reg.r[24] = if ms.reg.r[xlat!(rx)] < zero_ext8!(imm8) { 1 }else{ 0 };
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_I8 => {
            match rx {
                0 => {
                    if with_prefix!(inst32) {
                        if ms.emu.debug { info!("bteqz 0x{:x}(={:x})\n", imm16, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4); }
                        if ms.reg.r[24] == 0 {
                            update_pc_imm!(ms, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4);
                        }else{
                            update_pc_next32!(ms);
                        }
                    }else{
                        if ms.emu.debug { info!("bteqz 0x{:x}(={:x})\n", imm8, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2); }
                        if ms.reg.r[24] == 0 {
                            update_pc_imm!(ms, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2);
                        }else{
                            update_pc_next16!(ms);
                        }
                    }
                }
                1 => {
                    if with_prefix!(inst32) {
                        if ms.emu.debug { info!("btnez 0x{:x}(={:x})\n", imm16, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4); }
                        if ms.reg.r[24] != 0 {
                            update_pc_imm!(ms, ms.reg.pc + (sign_ext16!(imm16) << 1) + 4);
                        }else{
                            update_pc_next32!(ms);
                        }
                    }else{
                        if ms.emu.debug { info!("btnez 0x{:x}(={:x})\n", imm8, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2); }
                        if ms.reg.r[24] != 0 {
                            update_pc_imm!(ms, ms.reg.pc + (sign_ext8!(imm8) << 1) + 2);
                        }else{
                            update_pc_next16!(ms);
                        }
                    }
                }
                2 => /* SWRASP */ {
                    if with_prefix!(inst32) {
                        utmp = ms.reg.r[29] + sign_ext16!(imm16);
                        if ms.emu.debug { info!("sw ra, 0x{:x}(sp) (=0x{:x})", imm16, utmp); }
                        match mem::store_word(ms, utmp, ms.reg.r[31]) {
                            Ok(()) => { update_pc_next32!(ms); }
                            Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                        }
                    }else{
                        utmp = ms.reg.r[29] + (zero_ext8!(imm8)<<2);
                        if ms.emu.debug { info!("sw ra, 0x{:x}(sp) (=0x{:x})", (zero_ext8!(imm8)<<2), utmp); }
                        match mem::store_word(ms, utmp, ms.reg.r[31]) {
                            Ok(()) => { update_pc_next16!(ms); }
                            Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                        }
                    }
                }
                3 => /* ADJSP */ {
                    if with_prefix!(inst32) {
                        if ms.emu.debug { info!("addiu sp, 0x{:x}\n", imm16); }
                        ms.reg.r[29] += sign_ext16!(imm16);
                        update_pc_next32!(ms);
                    }else{
                        if ms.emu.debug { info!("addiu sp, 0x{:x}\n", sign_ext8!(imm8)<<3); }
                        ms.reg.r[29] += sign_ext8!(imm8)<<3;
                        update_pc_next16!(ms);
                    }
                }
                4 => {
                    if 0 != ( inst32 & (1<<7) ) {
                        // save function
                        let fs:u32 = inst32 & 0x0f;  // framesize
                        let  s1:bool = 0!=((inst32>>4) & 1);
                        let  s0:bool = 0!=((inst32>>5) & 1);
                        let  ra:bool = 0!=((inst32>>6) & 1);

                        if with_prefix!(inst32) {
                            // SAVE (Extended)
                            let aregs:u32  = (inst32>>16) & 0xf;
                            let xsregs:u32 = (inst32>>24) & 0x7;
                            let fs = fs | ((inst32>>16) & 0xf0);

                            if ms.emu.debug { info!("save ra={}, xsregs={}, aregs={}, s0={}, s1={}, framesize={}\n", ra, xsregs, aregs, s0,s1,fs); }

                            utmp  = ms.reg.r[29]; 
                            utmp2 = ms.reg.r[29];
                            let args : u32;
                            match aregs {
                                0| 1| 2| 3| 11 => { args = 0; }
                                4| 5| 6| 7 => { args = 1; }
                                8| 9| 10 => { args = 2; }
                                12| 13 => { args = 3; }
                                14 => { args = 4; }
                                _  => { args = 0; }
                            }
                            for idx in 0..args {
                                match mem::store_word(ms, utmp+(idx<<2), ms.reg.r[4+idx as usize]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp+(idx<<2));
                                        return true;
                                    }
                                }
                            }
                            if ra {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[31]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            for idx in (1..=xsregs).rev() {
                                let regnum:usize = if idx==7 { 30 }else{ 18 + (idx-1) as usize };
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[regnum]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s1 {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[17]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s0 {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[16]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            let astatic : u32;
                            match aregs {
                                0| 4| 8| 12| 14 => { astatic = 0; }
                                1| 5| 9| 13 => { astatic = 1; }
                                2| 6| 10 => { astatic = 2; }
                                3| 7 => { astatic = 3; }
                                11 => { astatic = 4; }
                                _  => { astatic = 0; }
                            }
                            for idx in 0..astatic {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[7-idx as usize]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            utmp = utmp2 - (fs<<3);
                            ms.reg.r[29] = utmp;

                            update_pc_next32!(ms);
                        }else{
                            // SAVE (non-extended)
                            if ms.emu.debug { info!("save ra={}, s0={}, s1={}, framesize={}\n", ra,s0,s1,fs); }

                            utmp  = ms.reg.r[29]; 
                            utmp2 = ms.reg.r[29];

                            if ra {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[31]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s1 {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[17]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s0 {
                                utmp -= 4;
                                match mem::store_word(ms, utmp, ms.reg.r[16]) {
                                    Ok(()) => {  }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if fs == 0 {
                                utmp = utmp2 - 128;
                            }else{
                                utmp = utmp2 - (fs<<3);
                            }
                            ms.reg.r[29] = utmp;

                            update_pc_next16!(ms);
                        }
                        return true;
                    }else{
                        // restore function
                        let fs:u32 = inst32 & 0x0f;  // framesize
                        let s1:bool = 0!=((inst32>>4) & 1);
                        let s0:bool = 0!=((inst32>>5) & 1);
                        let ra:bool = 0!=((inst32>>6) & 1);

                        if with_prefix!(inst32) {
                            // restore (Extended)

                            let aregs:u32  = (inst32>>16) & 0xf;
                            let xsregs:u32 = (inst32>>24) & 0x7;
                            let fs:u32 = fs | ((inst32>>16) & 0xf0);

                            if ms.emu.debug { info!("restore ra={}, xsregs={}, aregs={}, s0={}, s1={}, framesize={}\n", ra, xsregs, aregs, s0,s1,fs); }

                            utmp2 = ms.reg.r[29] + (fs <<3); 
                            utmp  = ms.reg.r[29] + (fs <<3);

                            if ra {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[31] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            for idx in (1..=xsregs).rev() {
                                let regnum :usize= if idx==7 { 30 }else{ 18 + (idx-1) as usize };
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[regnum] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s1 {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[17] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s0 {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[16] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            let astatic : u32;
                            match aregs {
                                0| 4| 8| 12| 14 => { astatic = 0; }
                                1| 5| 9| 13 => { astatic = 1; }
                                2| 6| 10 => { astatic = 2; }
                                3| 7 => { astatic = 3; }
                                11 => { astatic = 4; }
                                _  => { astatic = 0; }
                            }
                            for idx in 0..astatic {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[7-idx as usize] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            ms.reg.r[29] = utmp2;

                            update_pc_next32!(ms);
                        }else{
                            // restore (non-extended)

                            if ms.emu.debug { info!("restore ra={}, s0={}, s1={}, framesize={}\n", ra,s0,s1,fs); }

                            if fs == 0 {
                                utmp = ms.reg.r[29] + 128;
                            }else{
                                utmp = ms.reg.r[29] + (fs <<3);
                            }
                            utmp2 = utmp;

                            if ra {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[31] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s1 {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[17] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            if s0 {
                                utmp -= 4;
                                match mem::load_word(ms, utmp) {
                                    Ok(data) => { ms.reg.r[16] = data; }
                                    Err(ecode)  => { 
                                        exception::prepare_exception(ms, ecode, utmp);
                                        return true;
                                    }
                                }
                            }
                            ms.reg.r[29] = utmp2;

                            update_pc_next16!(ms);
                        }
                        return true;
                    }
                }
                5 => {
                    // This instruction has no extended version
                    utmp = ry | (funct5&0x18);
                    utmp2= funct5 & 0x7;
                    if ms.emu.debug { info!("move {}, {}\n", mips::REGSTR[utmp as usize], mips::REGSTR[xlat!(utmp2)]); }
                    ms.reg.r[utmp as usize] = ms.reg.r[xlat!(utmp2)];
                    update_pc_next16!(ms);
                }
                7 => {
                    // This instruction has no extended version
                    if ms.emu.debug { info!("move {}, {}\n", mips::REGSTR[xlat!(ry)], mips::REGSTR[funct5 as usize]); }
                    ms.reg.r[xlat!(ry)] = ms.reg.r[funct5 as usize];
                    update_pc_next16!(ms);
                }
                _ => {
                    unknown_instruction16!(inst32, "unhandled MIPS16 op (I8)");
                }
            }
        }
        MIPS16E_OP_LI => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("li {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[xlat!(rx)] = zero_ext16!(imm16);
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("li {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm8); }
                ms.reg.r[xlat!(rx)] = zero_ext8!(imm8);
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_CMPI => {
            if with_prefix!(inst32) {
                if ms.emu.debug { info!("cmpi {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm16); }
                ms.reg.r[24] = ms.reg.r[xlat!(rx)] ^ zero_ext16!(imm16);
                update_pc_next32!(ms);
            }else{
                if ms.emu.debug { info!("cmpi {}, 0x{:x}\n", mips::REGSTR[xlat!(rx)], imm8); }
                ms.reg.r[24] = ms.reg.r[xlat!(rx)] ^ zero_ext8!(imm8);
                update_pc_next16!(ms);
            }
        }
        MIPS16E_OP_LB => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("lb {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_byte(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = sign_ext8!(data); update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + funct5;
                if ms.emu.debug { info!("lb {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_byte(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = sign_ext8!(data); update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_LH => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("lh {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_halfword(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = sign_ext16!(data); update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + (funct5<<1);
                if ms.emu.debug { info!("lh {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5<<1, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_halfword(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = sign_ext16!(data); update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_LWSP => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[29] + sign_ext16!(imm16);
                if ms.emu.debug { info!("lw {}, 0x{:x}(sp) (=0x{:x})", mips::REGSTR[xlat!(rx)], imm16, utmp); }
                match mem::load_word(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(rx)] = data; update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[29] + (zero_ext8!(imm8)<<2);
                if ms.emu.debug { info!("lw {}, 0x{:x}(sp) (=0x{:x})", mips::REGSTR[xlat!(rx)], (zero_ext8!(imm8)<<2), utmp); }
                match mem::load_word(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(rx)] = data; update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_LW => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("lw {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_word(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = data; update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + (funct5<<2);
                if ms.emu.debug { info!("lw {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5<<2, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_word(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = data; update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_LBU => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("lbu {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_byte(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = zero_ext8!(data); update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + funct5;
                if ms.emu.debug { info!("lbu {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_byte(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = zero_ext8!(data); update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_LHU => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("lhu {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_halfword(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = zero_ext16!(data); update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + (funct5<<1);
                if ms.emu.debug { info!("lhu {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5<<1, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::load_halfword(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(ry)] = zero_ext16!(data); update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_LWPC => {
            if with_prefix!(inst32) {
                utmp = (ms.reg.pc & !(3 as u32)) + sign_ext16!(imm16);
                if ms.emu.debug { info!("lw {}, 0x{:x}(pc) (=0x{:x})", mips::REGSTR[xlat!(rx)], imm16, utmp); }
                match mem::load_word(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(rx)] = data; update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp =  ((if ms.reg.delay_en { ms.reg.pc_prev_jump }else{ ms.reg.pc}) & !(3 as u32)) + (zero_ext8!(imm8)<<2);
                if ms.emu.debug { info!("lw {}, 0x{:x}(pc) (=0x{:x})", mips::REGSTR[xlat!(rx)], (zero_ext8!(imm8)<<2), utmp); }
                match mem::load_word(ms, utmp) {
                    Ok(data) => { ms.reg.r[xlat!(rx)] = data; update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_SB => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("sb {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::store_byte(ms, utmp, ms.reg.r[xlat!(ry)] as u8) {
                    Ok(()) => { update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + funct5;
                if ms.emu.debug { info!("sb {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::store_byte(ms, utmp, ms.reg.r[xlat!(ry)] as u8) {
                    Ok(()) => { update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_SH => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("sh {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::store_halfword(ms, utmp, ms.reg.r[xlat!(ry)]) {
                    Ok(()) => { update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + (funct5<<1);
                if ms.emu.debug { info!("sh {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5<<1, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::store_halfword(ms, utmp, ms.reg.r[xlat!(ry)]) {
                    Ok(()) => { update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_SWSP => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[29] + sign_ext16!(imm16);
                if ms.emu.debug { info!("sw {}, 0x{:x}(sp) (=0x{:x})", mips::REGSTR[xlat!(rx)], imm16, utmp); }
                match mem::store_word(ms, utmp, ms.reg.r[xlat!(rx)]) {
                    Ok(()) => { update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[29] + (zero_ext8!(imm8)<<2);
                if ms.emu.debug { info!("sw {}, 0x{:x}(sp) (=0x{:x})", mips::REGSTR[xlat!(rx)], (zero_ext8!(imm8)<<2), utmp); }
                match mem::store_word(ms, utmp, ms.reg.r[xlat!(rx)]) {
                    Ok(()) => { update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_SW => {
            if with_prefix!(inst32) {
                utmp = ms.reg.r[xlat!(rx)] + sign_ext16!(imm16);
                if ms.emu.debug { info!("sw {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], imm16, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::store_word(ms, utmp, ms.reg.r[xlat!(ry)]) {
                    Ok(()) => { update_pc_next32!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }else{
                utmp = ms.reg.r[xlat!(rx)] + (funct5<<2);
                if ms.emu.debug { info!("sw {}, 0x{:x}({}) (=0x{:x})", mips::REGSTR[xlat!(ry)], funct5<<2, mips::REGSTR[xlat!(rx)], utmp); }
                match mem::store_word(ms, utmp, ms.reg.r[xlat!(ry)]) {
                    Ok(()) => { update_pc_next16!(ms); }
                    Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
                }
            }
        }
        MIPS16E_OP_RRR => {
            match inst32 & 3 {
                1 => {
                    if ms.emu.debug { info!("addu {}, {}, {}\n", mips::REGSTR[xlat!(rz)], mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rz)] = ms.reg.r[xlat!(rx)] + ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                3 => {
                    if ms.emu.debug { info!("subu {}, {}, {}\n", mips::REGSTR[xlat!(rz)], mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rz)] = ms.reg.r[xlat!(rx)] - ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                _ => {
                    unknown_instruction16!(inst32, "unhandled MIPS16 op (RRR)");
                }
            }
        }
        MIPS16E_OP_RR => {
            let jumpaddr : u32;
            match funct5 {
                MIPS16E_RRFUNCT_JR => {
                    match ry {
                        2 => {
                            jumpaddr = ms.reg.r[xlat!(rx)];
                            ms.reg.r[31] = ms.reg.pc + 4;
                            update_pc_next16_with_delayed_imm!(ms, jumpaddr);
                            if ms.emu.debug { info!("jalr {}(=0x{:x})\n", mips::REGSTR[xlat!(rx)], jumpaddr); }
                        }
                        6 => {
                            jumpaddr = ms.reg.r[xlat!(rx)];
                            ms.reg.r[31] = ms.reg.pc + 2;
                            update_pc_imm!(ms, jumpaddr);
                            if ms.emu.debug { info!("jalrc {}(=0x{:x})\n", mips::REGSTR[xlat!(rx)], jumpaddr); }
                        }
                        4 => {
                            update_pc_imm!(ms, ms.reg.r[xlat!(rx)]);
                            if ms.emu.debug { info!("jrc {}(=0x{:x})\n", mips::REGSTR[xlat!(rx)], ms.reg.r[xlat!(rx)]); }
                        }
                        5 => {
                            update_pc_imm!(ms, ms.reg.r[31]);
                            if ms.emu.debug { info!("jrc ra(=0x{:x})\n", ms.reg.r[31]); }
                        }
                        0 => {
                            update_pc_next16_with_delayed_imm!(ms, ms.reg.r[xlat!(rx)]);
                            if ms.emu.debug { info!("jr {}(=0x{:x})\n", mips::REGSTR[xlat!(rx)], ms.reg.r[xlat!(rx)]); }
                        }
                        1 => {
                            update_pc_next16_with_delayed_imm!(ms, ms.reg.r[31]);
                            if ms.emu.debug { info!("jr ra(=0x{:x})\n", ms.reg.r[31]); }
                        }
                        _ => {
                            unknown_instruction16!(inst32, "unhandled MIPS16 op (JR)");
                        }
                    }
                }
                MIPS16E_RRFUNCT_SDBBP => {
                    /* Software Debug Breakpoint */
                    unknown_instruction16!(inst32, "unhandled MIPS16 op (SDBBP)");
                }
                MIPS16E_RRFUNCT_SLT => {
                    if ms.emu.debug { info!("slt {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[24] = if (ms.reg.r[xlat!(rx)] as i32) < (ms.reg.r[xlat!(ry)] as i32) { 1 }else{ 0 };
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_SLTU => {
                    if ms.emu.debug { info!("sltu {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[24] = if ms.reg.r[xlat!(rx)] < ms.reg.r[xlat!(ry)] { 1 }else{ 0 };
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_SLLV => {
                    if ms.emu.debug { info!("sllv {}, {}\n", mips::REGSTR[xlat!(ry)], mips::REGSTR[xlat!(rx)]); }
                    ms.reg.r[xlat!(ry)] = ms.reg.r[xlat!(ry)] << ((ms.reg.r[xlat!(rx)]) & 0x1f);
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_BREAK => {
                    unknown_instruction16!(inst32, "unhandled MIPS16 op (BREAK)");
                }
                MIPS16E_RRFUNCT_SRLV => {
                    if ms.emu.debug { info!("srlv {}, {}\n", mips::REGSTR[xlat!(ry)], mips::REGSTR[xlat!(rx)]); }
                    ms.reg.r[xlat!(ry)] = (((ms.reg.r[xlat!(ry)] as u32) as u64) >> ((ms.reg.r[xlat!(rx)]) & 0x1f)) as u32;
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_SRAV => {
                    if ms.emu.debug { info!("srav {}, {}\n", mips::REGSTR[xlat!(ry)], mips::REGSTR[xlat!(rx)]); }
                    ms.reg.r[xlat!(ry)] = (((ms.reg.r[xlat!(ry)] as i32) as i64) >> ((ms.reg.r[xlat!(rx)]) & 0x1f)) as u32;
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_CMP => {
                    if ms.emu.debug { info!("cmp {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[24] = ms.reg.r[xlat!(rx)] ^ ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_NEG => {
                    if ms.emu.debug { info!("neg {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rx)] = 0 - ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_AND => {
                    if ms.emu.debug { info!("and {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rx)] = ms.reg.r[xlat!(rx)] & ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_OR => {
                    if ms.emu.debug { info!("or {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rx)] = ms.reg.r[xlat!(rx)] | ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_XOR => {
                    if ms.emu.debug { info!("xor {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rx)] = ms.reg.r[xlat!(rx)] ^ ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_NOT => {
                    if ms.emu.debug { info!("not {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    ms.reg.r[xlat!(rx)] = !ms.reg.r[xlat!(ry)];
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_MFHI => {
                    if ms.emu.debug { info!("mfhi {}\n", mips::REGSTR[xlat!(rx)]); }
                    ms.reg.r[xlat!(rx)] = ms.reg.hi;
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_CNVT => {
                    match ry {
                        0 => {
                            if ms.emu.debug { info!("zeb {}\n", mips::REGSTR[xlat!(rx)]); }
                            ms.reg.r[xlat!(rx)] = (ms.reg.r[xlat!(rx)] as u8) as u32;
                            update_pc_next16!(ms);
                        }
                        1 => {
                            if ms.emu.debug { info!("zeh {}\n", mips::REGSTR[xlat!(rx)]); }
                            ms.reg.r[xlat!(rx)] = (ms.reg.r[xlat!(rx)] as u16) as u32;
                            update_pc_next16!(ms);
                        }
                        4 => {
                            if ms.emu.debug { info!("seb {}\n", mips::REGSTR[xlat!(rx)]); }
                            ms.reg.r[xlat!(rx)] = ((ms.reg.r[xlat!(rx)] as i8) as i32) as u32;
                            update_pc_next16!(ms);
                        }
                        5 => {
                            if ms.emu.debug { info!("seh {}\n", mips::REGSTR[xlat!(rx)]); }
                            ms.reg.r[xlat!(rx)] = ((ms.reg.r[xlat!(rx)] as i16) as i32) as u32;
                            update_pc_next16!(ms);
                        }
                        _ => {
                            unknown_instruction16!(inst32, "unhandled MIPS16 op (CNVT)");
                        }
                    }
                }
                MIPS16E_RRFUNCT_MFLO => {
                    if ms.emu.debug { info!("mflo {}\n", mips::REGSTR[xlat!(rx)]); }
                    ms.reg.r[xlat!(rx)] = ms.reg.lo;
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_MULT => {
                    if ms.emu.debug { info!("mult {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    let mul_tmp:i64 = (((ms.reg.r[xlat!(rx)] as i32) as i64)) * (((ms.reg.r[xlat!(ry)] as i32) as i64));
                    ms.reg.hi = ((mul_tmp>>16)>>16) as u32;
                    ms.reg.lo = mul_tmp as u32;
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_MULTU => {
                    if ms.emu.debug { info!("multu {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    let mul_tmp : u64 = ((ms.reg.r[xlat!(rx)] as u32) as u64) * ((ms.reg.r[xlat!(ry)] as u32) as u64);
                    ms.reg.hi = ((mul_tmp>>16)>>16) as u32;
                    ms.reg.lo = mul_tmp as u32;
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_DIV => {
                    if ms.emu.debug { info!("div {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    if ms.reg.r[xlat!(ry)] == 0 {
                        // zero division
                        ms.reg.lo = 0; // q
                        ms.reg.hi = 0; // r
                    }else{
                        ms.reg.lo = ((ms.reg.r[xlat!(rx)] as i32) / (ms.reg.r[xlat!(ry)] as i32)) as u32; // q
                        ms.reg.hi = ((ms.reg.r[xlat!(rx)] as i32) % (ms.reg.r[xlat!(ry)] as i32)) as u32; // r
                    }
                    update_pc_next16!(ms);
                }
                MIPS16E_RRFUNCT_DIVU => {
                    if ms.emu.debug { info!("divu {}, {}\n", mips::REGSTR[xlat!(rx)], mips::REGSTR[xlat!(ry)]); }
                    if ms.reg.r[xlat!(ry)] == 0 {
                        // zero division
                        ms.reg.lo = 0; // q
                        ms.reg.hi = 0; // r
                    }else{
                        ms.reg.lo = (ms.reg.r[xlat!(rx)] as u32) / (ms.reg.r[xlat!(ry)] as u32); // q
                        ms.reg.hi = (ms.reg.r[xlat!(rx)] as u32) % (ms.reg.r[xlat!(ry)] as u32); // r
                    }
                    update_pc_next16!(ms);
                }
                _ => {
                    unknown_instruction16!(inst32, "unhandled MIPS16 op (RR)");
                }
            }
        }
        MIPS16E_OP_EXTEND => {
            /* not processed here */
            unknown_instruction16!(inst32, "unhandled MIPS16 op (EXTEND)");
        }
        _ => {
            unknown_instruction16!(inst32, "unhandled MIPS16 op");
        }
    }

    true
}