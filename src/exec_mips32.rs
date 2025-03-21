use crate::procstate::MachineState;
use crate::exception;
use crate::config;
use crate::cp0def;
use crate::mips;
use crate::mode_is_exception;
use crate::mode_is_user;
use crate::tlb;
use crate::cp0;
use crate::mem;

//use crate::exec_common;
use crate::sign_ext16;
use crate::zero_ext16;
use crate::c0_val;
use crate::update_pc_next32;
use crate::update_pc_next32_with_delayed_imm;
use log::{error,info};
use std::thread;
use std::time::Duration;

macro_rules! unknown_instruction{
    ( $inst:expr, $msg:expr ) => 
    {
        error!("Unknown MIPS32 instruction (inst={:>08x}, {})", $inst, $msg);
        return false;
    }
}

pub const MIPS32_OP_SPECIAL : u32 = 0b000_000;
pub const MIPS32_OP_REGIMM  : u32 = 0b000_001;
pub const MIPS32_OP_J       : u32 = 0b000_010;
pub const MIPS32_OP_JAL     : u32 = 0b000_011;
pub const MIPS32_OP_BEQ     : u32 = 0b000_100;
pub const MIPS32_OP_BNE     : u32 = 0b000_101;
pub const MIPS32_OP_BLEZ    : u32 = 0b000_110;
pub const MIPS32_OP_BGTZ    : u32 = 0b000_111;
pub const MIPS32_OP_ADDI    : u32 = 0b001_000;
pub const MIPS32_OP_ADDIU   : u32 = 0b001_001;
pub const MIPS32_OP_SLTI    : u32 = 0b001_010;
pub const MIPS32_OP_SLTIU   : u32 = 0b001_011;
pub const MIPS32_OP_ANDI    : u32 = 0b001_100;
pub const MIPS32_OP_ORI     : u32 = 0b001_101;
pub const MIPS32_OP_XORI    : u32 = 0b001_110;
pub const MIPS32_OP_LUI     : u32 = 0b001_111;
pub const MIPS32_OP_COP0    : u32 = 0b010_000;
pub const MIPS32_OP_COP1    : u32 = 0b010_001;
pub const MIPS32_OP_COP2    : u32 = 0b010_010;
pub const MIPS32_OP_COP1X   : u32 = 0b010_011;
pub const MIPS32_OP_BEQL    : u32 = 0b010_100;
pub const MIPS32_OP_BNEL    : u32 = 0b010_101;
pub const MIPS32_OP_BLEZL   : u32 = 0b010_110;
pub const MIPS32_OP_BGTZL   : u32 = 0b010_111;
//pub const MIPS32_OP_POP30   : u32 = 0b011_000;
pub const MIPS32_OP_SPECIAL2: u32 = 0b011_100;
pub const MIPS32_OP_JALX    : u32 = 0b011_101;
pub const MIPS32_OP_MSA     : u32 = 0b011_110;
pub const MIPS32_OP_SPECIAL3: u32 = 0b011_111;
pub const MIPS32_OP_LB      : u32 = 0b100_000;
pub const MIPS32_OP_LH      : u32 = 0b100_001;
pub const MIPS32_OP_LWL     : u32 = 0b100_010;
pub const MIPS32_OP_LW      : u32 = 0b100_011;
pub const MIPS32_OP_LBU     : u32 = 0b100_100;
pub const MIPS32_OP_LHU     : u32 = 0b100_101;
pub const MIPS32_OP_LWR     : u32 = 0b100_110;
pub const MIPS32_OP_SB      : u32 = 0b101_000;
pub const MIPS32_OP_SH      : u32 = 0b101_001;
pub const MIPS32_OP_SWL     : u32 = 0b101_010;
pub const MIPS32_OP_SW      : u32 = 0b101_011;
pub const MIPS32_OP_SWR     : u32 = 0b101_110;
pub const MIPS32_OP_CACHE   : u32 = 0b101_111;
pub const MIPS32_OP_LL      : u32 = 0b110_000;
pub const MIPS32_OP_LWC1    : u32 = 0b110_001;
pub const MIPS32_OP_LWC2    : u32 = 0b110_010;
pub const MIPS32_OP_PREF    : u32 = 0b110_011;
pub const MIPS32_OP_LDC1    : u32 = 0b110_101;
pub const MIPS32_OP_LDC2    : u32 = 0b110_110;
pub const MIPS32_OP_SC      : u32 = 0b111_000;
pub const MIPS32_OP_SWC1    : u32 = 0b111_001;
pub const MIPS32_OP_SWC2    : u32 = 0b111_010;
pub const MIPS32_OP_PCREL   : u32 = 0b111_011;
pub const MIPS32_OP_SDC1    : u32 = 0b111_101;
pub const MIPS32_OP_SDC2    : u32 = 0b111_110;



pub fn exec(ms: &mut MachineState, inst : u32) -> bool {
    let pointer : u32 = ms.reg.pc;

    let op    : u32 = (inst>>26) & 0x3f; /* inst[31:26] */
    let rs    : usize = ((inst>>21) & 0x1f) as usize; /* inst[25:21] */
    let rt    : usize = ((inst>>16) & 0x1f) as usize; /* inst[20:16] */
    let rd    : usize = ((inst>>11) & 0x1f) as usize; /* inst[15:11] */
    let rd_u32: u32 = (inst>>11) & 0x1f;   /* inst[15:11] */
    let shamt : u32 = (inst>> 6) & 0x1f;   /* inst[10: 6] */
    let funct : u32 =  inst      & 0x3f;   /* inst[ 0: 5] */
    let imm   : u32 =  inst      & 0xffff; /* inst[15: 0] */

    let mut utmp:u32;
    let jumpaddr :u32;
    let loaddata :u32;

    if ms.emu.debug {
        info!("cnt:{} PC:{:>08x}  \r", ms.emu.nexec_insts, pointer);
    }


    if inst == 0 {
        if ms.emu.debug { info!("nop"); }
        update_pc_next32!(ms);
        return true;
    }

    match op {
        MIPS32_OP_SPECIAL =>
        {
            match funct {
                0x20 => // add 
                {
                    if ms.emu.debug { info!("add {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    match (ms.reg.r[rs] as i32).overflowing_add(ms.reg.r[rt] as i32){
                        (res, true ) => { ms.reg.r[rd] = res as u32; exception::prepare_exception(ms, cp0def::EXCEPT_CODE_INTEGER_OVERFLOW, 0); }
                        (res, false) => { ms.reg.r[rd] = res as u32; update_pc_next32!(ms); }
                    }
                    return true;
                }
                0x21 => // addu
                {
                    if ms.emu.debug { info!("addu {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = ms.reg.r[rs] + ms.reg.r[rt];
                    update_pc_next32!(ms);
                    return true;
                }
                0x24 => // and
                {
                    if ms.emu.debug { info!("and {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = ms.reg.r[rs] & ms.reg.r[rt];
                    update_pc_next32!(ms);
                    return true;
                }
                0x08 => // jr
                {
                    update_pc_next32_with_delayed_imm!(ms, ms.reg.r[rs]);
                    if ms.emu.debug { info!("jr {}(=0x{:>x})", mips::REGSTR[rs], ms.reg.r[rs]); }
                    return true;
                }
                0x09 => // jalr
                {
                    if ms.emu.debug { info!("jalr {}, {}(=0x{:>x})", mips::REGSTR[rd], mips::REGSTR[rs], ms.reg.r[rs]); }
                    jumpaddr = ms.reg.r[rs];
                    ms.reg.r[rd] = ms.reg.pc + 8;
                    update_pc_next32_with_delayed_imm!(ms, jumpaddr);
                    return true;
                }
                0x27 => // nor
                {
                    if ms.emu.debug { info!("nor {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = !(ms.reg.r[rs] | ms.reg.r[rt]);
                    update_pc_next32!(ms);
                    return true;
                }
                0x25 => // or
                {
                    if ms.emu.debug { info!("or {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = ms.reg.r[rs] | ms.reg.r[rt];
                    update_pc_next32!(ms);
                    return true;
                }
                0x26 => // xor
                {
                    if ms.emu.debug { info!("xor {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = ms.reg.r[rs] ^ ms.reg.r[rt];
                    update_pc_next32!(ms);
                    return true;
                }
                0x2a => // slt
                {
                    if ms.emu.debug { info!("slt {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = if (ms.reg.r[rs] as i32) < (ms.reg.r[rt] as i32) { 1 }else{ 0 };
                    update_pc_next32!(ms);
                    return true;
                }
                0x2b => // sltu
                {
                    if ms.emu.debug { info!("sltu {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = if (ms.reg.r[rs] as u32) < (ms.reg.r[rt] as u32) { 1 } else{ 0 };
                    update_pc_next32!(ms);
                    return true;
                }
                0x0a => // movz
                {
                    if ms.emu.debug { info!("movz {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    if ms.reg.r[rt] == 0 {
                        ms.reg.r[rd] = ms.reg.r[rs];
                    }
                    update_pc_next32!(ms);
                    return true;
                }
                0x0b => // movn
                {
                    if ms.emu.debug { info!("movn {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    if ms.reg.r[rt] != 0 {
                        ms.reg.r[rd] = ms.reg.r[rs];
                    }
                    update_pc_next32!(ms);
                    return true;
                }
                0x00 => // sll
                {
                    if inst == 0x00000000 {
                        if ms.emu.debug { info!("nop"); }
                        update_pc_next32!(ms);
                    }else if inst == 0x000000c0 {
                        if ms.emu.debug { info!("ehb"); }
                        update_pc_next32!(ms);
                    }else if rs == 0x00 {
                        if ms.emu.debug { info!("sll {}, {}, 0x{:>x}", mips::REGSTR[rd], mips::REGSTR[rt], shamt); }
                        ms.reg.r[rd] = ms.reg.r[rt] << shamt;
                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"op=0x00 funct=0x00");
                    }
                    return true;
                }
                0x02 => // srl or rotr
                {
                    if rs == 0 { // srl
                        if ms.emu.debug { info!("srl {}, {}, 0x{:>x}", mips::REGSTR[rd], mips::REGSTR[rt], shamt); }
                        ms.reg.r[rd] = (((ms.reg.r[rt] as u32) as u64) >> shamt) as u32;
                        update_pc_next32!(ms);

                    }else if rs == 1 { // rotr
                        if ms.emu.debug { info!("rotr {}, {}, 0x{:>x}", mips::REGSTR[rd], mips::REGSTR[rt], shamt); }
                        if shamt != 0 {
                            utmp = (((ms.reg.r[rt] as u32) as u64) >> shamt) as u32;
                            utmp|=   (ms.reg.r[rt] as u32) << (32-shamt);
                        }else{
                            utmp = ms.reg.r[rt];
                        }
                        ms.reg.r[rd] = utmp;
                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"right shift 0x02");
                    }
                    return true;
                }
                0x03 => // sra
                {
                    if ms.emu.debug { info!("sra {}, {}, 0x{:>x}", mips::REGSTR[rd], mips::REGSTR[rt], shamt); }
                    ms.reg.r[rd] = (((ms.reg.r[rt] as i32) as i64) >> shamt) as u32;
                    update_pc_next32!(ms);
                    return true;
                }
                0x04 => // sllv
                {
                    if ms.emu.debug { info!("sllv {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rt], mips::REGSTR[rs]); }
                    ms.reg.r[rd] = ms.reg.r[rt] << ((ms.reg.r[rs]) & 0x1f);
                    update_pc_next32!(ms);
                    return true;
                }
                0x06 => // srlv or rotrv
                {
                    if shamt == 0 { // srlv
                        if ms.emu.debug { info!("srlv {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rt], mips::REGSTR[rs]); }
                        ms.reg.r[rd] = (((ms.reg.r[rt] as u32) as u64) >> ((ms.reg.r[rs]) & 0x1f)) as u32;
                        update_pc_next32!(ms);
                    }else if shamt == 1 { // rotrv
                        if ms.emu.debug { info!("rotrv {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rt], mips::REGSTR[rs]); }
                        let sa: u32 = ms.reg.r[rt] & 0x1f;
                        if sa != 0 {
                            utmp = (((ms.reg.r[rt] as u32) as u64) >> sa) as u32;
                            utmp|=   (ms.reg.r[rt] as u32) << (32-sa);
                        }else{
                            utmp = ms.reg.r[rt];
                        }
                        ms.reg.r[rd] = utmp;
                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"right shift 0x06");
                    }
                    return true;
                }
                0x07 => // srav
                {
                    if shamt == 0 {
                        if ms.emu.debug { info!("srav {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rt], mips::REGSTR[rs]); }
                        ms.reg.r[rd] = (((ms.reg.r[rt] as i32) as i64) >> ((ms.reg.r[rs]) & 0x1f)) as u32;
                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"right shift 0x07");
                    }
                    return true;
                }
                0x22 => // sub
                {
                    if ms.emu.debug { info!("sub {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    match (ms.reg.r[rs] as i32).overflowing_sub( ms.reg.r[rt] as i32) {
                        (res, true ) => { ms.reg.r[rd] = res as u32; exception::prepare_exception(ms, cp0def::EXCEPT_CODE_INTEGER_OVERFLOW, 0); }
                        (res, false) => { ms.reg.r[rd] = res as u32; update_pc_next32!(ms); }
                    }
                    return true;
                }
                0x23 => // subu
                {
                    if ms.emu.debug { info!("subu {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                    ms.reg.r[rd] = ms.reg.r[rs] - ms.reg.r[rt];
                    update_pc_next32!(ms);
                    return true;
                }
                0x10 => // mfhi
                {
                    if ms.emu.debug { info!("mfhi {}", mips::REGSTR[rd]); }
                    ms.reg.r[rd] = ms.reg.hi;
                    update_pc_next32!(ms);
                    return true;
                }
                0x11 => // mthi
                {
                    if ms.emu.debug { info!("mthi {}", mips::REGSTR[rs]); }
                    ms.reg.hi = ms.reg.r[rs];
                    update_pc_next32!(ms);
                    return true;
                }
                0x12 => // mflo
                {
                    if ms.emu.debug { info!("mflo {}", mips::REGSTR[rd]); }
                    ms.reg.r[rd] = ms.reg.lo;
                    update_pc_next32!(ms);
                    return true;
                }
                0x13 => // mtlo
                {
                    if ms.emu.debug { info!("mtlo {}", mips::REGSTR[rs]); }
                    ms.reg.lo = ms.reg.r[rs];
                    update_pc_next32!(ms);
                    return true;
                }
                0x18 =>
                {
                    if shamt == 0x00 && rd == 0x00 {
                        if ms.emu.debug { info!("mult {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                        let mul_tmp: i64 = ((ms.reg.r[rs] as i32) as i64) * ((ms.reg.r[rt] as i32) as i64);
                        ms.reg.hi = ((mul_tmp>>16)>>16) as u32;
                        ms.reg.lo = mul_tmp as u32;
                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"op=0x00, funct=0x19");
                    }
                    return true;
                }
                0x19 =>
                {
                    if shamt == 0x00 && rd == 0x00 {
                        if ms.emu.debug { info!("multu {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                        let mul_tmp: u64 = ((ms.reg.r[rs] as u32) as u64) * ((ms.reg.r[rt] as u32) as u64);
                        ms.reg.hi = ((mul_tmp>>16)>>16) as u32;
                        ms.reg.lo = mul_tmp as u32;
                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"op=0x00, funct=0x19");
                    }
                    return true;
                }
                0x1a => // div
                {
                    if ms.emu.debug { info!("div {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                    if ms.reg.r[rt] == 0 {
                        // zero division
                        ms.reg.lo = 0; // q
                        ms.reg.hi = 0; // r
                    }else{
                        ms.reg.lo = ((ms.reg.r[rs] as i32) / (ms.reg.r[rt] as i32)) as u32; // q
                        ms.reg.hi = ((ms.reg.r[rs] as i32) % (ms.reg.r[rt] as i32)) as u32; // r
                    }
                    update_pc_next32!(ms);
                    return true;
                }
                0x1b => // divu
                {
                    if ms.emu.debug { info!("divu {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                    if ms.reg.r[rt] == 0 {
                        // zero division
                        ms.reg.lo = 0; // q
                        ms.reg.hi = 0; // r
                    }else{
                        ms.reg.lo = (ms.reg.r[rs] as u32) / (ms.reg.r[rt] as u32); // q
                        ms.reg.hi = (ms.reg.r[rs] as u32) % (ms.reg.r[rt] as u32); // r
                    }
                    update_pc_next32!(ms);
                    return true;
                }
                0x30 => // tge (trap if greater or equal)
                {
                    if ms.emu.debug { info!("tge {}, {}, 0x{:>x}", mips::REGSTR[rs], mips::REGSTR[rt], (inst>>6)&(0x3ff)); }
                    if (ms.reg.r[rs] as i32) >= (ms.reg.r[rt] as i32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                    return true;
                }
                0x31 => // tgeu (trap if greater or equal unsigned)
                {
                    if ms.emu.debug { info!("tgeu {}, {}, 0x{:>x}", mips::REGSTR[rs], mips::REGSTR[rt], (inst>>6)&(0x3ff)); }
                    if (ms.reg.r[rs] as u32) >= (ms.reg.r[rt] as u32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                    return true;
                }
                0x32 => // tlt (trap if less than)
                {
                    if ms.emu.debug { info!("tlt {}, {}, 0x{:>x}", mips::REGSTR[rs], mips::REGSTR[rt], (inst>>6)&(0x3ff)); }
                    if (ms.reg.r[rs] as i32) < (ms.reg.r[rt] as i32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                    return true;
                }
                0x33 => // tltu (trap if less than unsigned)
                {
                    if ms.emu.debug { info!("tltu {}, {}, 0x{:>x}", mips::REGSTR[rs], mips::REGSTR[rt], (inst>>6)&(0x3ff)); }
                    if (ms.reg.r[rs] as u32) < (ms.reg.r[rt] as u32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                    return true;
                }
                0x34 => // teq (trap if equal)
                {
                    if ms.emu.debug { info!("teq {}, {}, 0x{:>x}", mips::REGSTR[rs], mips::REGSTR[rt], (inst>>6)&(0x3ff)); }
                    if ms.reg.r[rs] == ms.reg.r[rt] {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                    return true;
                }
                0x36 => // tne (trap if not equal)
                {
                    if ms.emu.debug { info!("tne {}, {}, 0x{:>x}", mips::REGSTR[rs], mips::REGSTR[rt], (inst>>6)&(0x3ff)); }
                    if ms.reg.r[rs] != ms.reg.r[rt] {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                    return true;
                }
                0x0c => // syscall
                {
                    if ms.emu.debug { info!("syscall"); }
                    //printf("syscall v0:{:>x} a0:{:>x} a1:{:>x}", ms.reg.r[2], ms.reg.r[4], ms.reg.r[5]);
                    exception::prepare_exception(ms, cp0def::EXCEPT_CODE_SYSCALL, 0);
                    return true;
                }
                0x0f => // sync
                {
                    if ms.emu.debug { info!("sync {}", shamt); }
                    update_pc_next32!(ms);
                    return true;
                }
                _ =>
                {
                    unknown_instruction!(inst,"op=0x00");
                }
            }
        }
        MIPS32_OP_ADDI => // addi
        {
            if ms.emu.debug { info!("addi {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }

            match (ms.reg.r[rs] as i32).overflowing_add(sign_ext16!(imm) as i32) {
                ( res , false) => { ms.reg.r[rt] = res as u32; update_pc_next32!(ms); }
                ( res , true ) => { ms.reg.r[rt] = res as u32; exception::prepare_exception(ms, cp0def::EXCEPT_CODE_INTEGER_OVERFLOW, 0); }
            }
            return true;
        }
        MIPS32_OP_ADDIU => // addiu
        {
            if ms.emu.debug { info!("addiu {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }
            ms.reg.r[rt] = ms.reg.r[rs] + sign_ext16!(imm);
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_ANDI => // andi
        {
            if ms.emu.debug { info!("andi {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }
            ms.reg.r[rt] = ms.reg.r[rs] & zero_ext16!(imm);
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_BEQ => // beq
        {
            if ms.emu.debug { info!("beq {}, {}, 0x{:>x}(={:>x})", mips::REGSTR[rt], mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
            if ms.reg.r[rs] == ms.reg.r[rt] {
                update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
            }else{
                update_pc_next32!(ms);
            }
            return true;
        }
        MIPS32_OP_BEQL => // beql
        {
            if ms.emu.debug { info!("beql {}, {}, 0x{:>x}(={:>x})", mips::REGSTR[rt], mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
            if ms.reg.r[rs] == ms.reg.r[rt] {
                update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
            }else{
                update_pc_next32!(ms);
                update_pc_next32!(ms);
            }
            return true;
        }
        MIPS32_OP_BNE => // bne
        {
            if ms.emu.debug { info!("bne {}, {}, 0x{:>x}(={:>x})", mips::REGSTR[rt], mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
            if ms.reg.r[rs] != ms.reg.r[rt] {
                update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
            }else{
                update_pc_next32!(ms);
            }
            return true;
        }
        MIPS32_OP_BLEZ => // blez
        {
            if rt == 0 {
                if ms.emu.debug { info!("blez {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                if (ms.reg.r[rs] as i32) <= 0 {
                    update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                }else{
                    update_pc_next32!(ms);
                }
            }else{
                unknown_instruction!(inst,"op=0x06");
            }
            return true;
        }
        MIPS32_OP_BLEZL => // blezl
        {
            if rt == 0 {
                if ms.emu.debug { info!("blez {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                if (ms.reg.r[rs] as i32) <= 0 {
                    update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                }else{
                    update_pc_next32!(ms);
                    update_pc_next32!(ms);
                }
            }else{
                unknown_instruction!(inst,"op=0x16");
            }
            return true;
        }
        MIPS32_OP_BNEL => // bnel
        {
            if ms.emu.debug { info!("bnel {}, {}, 0x{:>x}(={:>x})", mips::REGSTR[rt], mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
            if ms.reg.r[rs] != ms.reg.r[rt] {
                update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
            }else{
                update_pc_next32!(ms);
                update_pc_next32!(ms);
            }
            return true;
        }
        MIPS32_OP_BGTZ => // bgtz
        {
            if ms.emu.debug { info!("bgtz {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
            if (ms.reg.r[rs] as i32) > 0 {
                update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
            }else{
                update_pc_next32!(ms);
            }
            return true;
        }
        MIPS32_OP_BGTZL => // bgtzl (changed in MIPS32R6)
        {
            if rt == 0x00 {
                if ms.emu.debug { info!("bgtzl {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                if (ms.reg.r[rs] as i32) > 0 {
                    update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                }else{
                    update_pc_next32!(ms);
                    update_pc_next32!(ms);
                }
            }else{
                unknown_instruction!(inst,"op=0x17");
            }
            return true;
        }
        MIPS32_OP_J => // j
        {
            jumpaddr    = inst & 0x03ffffff;
            update_pc_next32_with_delayed_imm!( ms, ((ms.reg.pc+4) & ((!0x03ffffff)<<2)) + (jumpaddr<<2) );
            if ms.emu.debug { info!("j 0x{:>x}(={:>x})", jumpaddr, ms.reg.pc_delay); }
            return true;
        }
        MIPS32_OP_JAL => // jal
        {
            jumpaddr      = inst & 0x03ffffff;
            ms.reg.r[31] = ms.reg.pc + 8;
            update_pc_next32_with_delayed_imm!( ms, ((ms.reg.pc+4) & ((!0x03ffffff)<<2)) + (jumpaddr<<2) );
            if ms.emu.debug { info!("jal 0x{:>x}(={:>x})", jumpaddr, ms.reg.pc_delay); }
            return true;
        }
        MIPS32_OP_JALX => // jalx
        {
            jumpaddr      = inst & 0x03ffffff;
            ms.reg.r[31] = ms.reg.pc + 8;
            update_pc_next32_with_delayed_imm!( ms, ((ms.reg.pc+4) & ((!0x03ffffff)<<2)) + (jumpaddr<<2) + 1 );
            if ms.emu.debug { info!("jalx 0x{:>x}(={:>x})", jumpaddr, ms.reg.pc_delay); }
            return true;
        }
        MIPS32_OP_REGIMM => // REGIMM
        {
            match rt {
                0x00 => // bltz
                {
                    if ms.emu.debug { info!("bltz {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                    if (ms.reg.r[rs] as i32) < 0 {
                        update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x02 => // bltzl
                {
                    if ms.emu.debug { info!("bltzl {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                    if (ms.reg.r[rs] as i32) < 0 {
                        update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                    }else{
                        update_pc_next32!(ms);
                        update_pc_next32!(ms);
                    }
                }
                0x10 => // bltzal
                {
                    if ms.emu.debug { info!("bltzal {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                    ms.reg.r[31] = ms.reg.pc + 8;
                    if (ms.reg.r[rs] as i32) < 0 {
                        update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x01 => // bgez
                {
                    if ms.emu.debug { info!("bgez {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                    if (ms.reg.r[rs] as i32) >= 0 {
                        update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x03 => // bgezl
                {
                    if ms.emu.debug { info!("bgezl {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                    if (ms.reg.r[rs] as i32) >= 0 {
                        update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                    }else{
                        update_pc_next32!(ms);
                        update_pc_next32!(ms);
                    }
                }
                0x11 => // bal, bgezal
                {
                    if rs == 0 { //bal
                        if ms.emu.debug { info!("bal 0x{:>x}(={:>x})", imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                        ms.reg.r[31] = ms.reg.pc + 8;
                        update_pc_next32_with_delayed_imm!( ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                    }else{ // bgezal
                        if ms.emu.debug { info!("bgezal {}, 0x{:>x}(={:>x})", mips::REGSTR[rs], imm, ms.reg.pc + (sign_ext16!(imm) << 2) + 4); }
                        ms.reg.r[31] = ms.reg.pc + 8;
                        if (ms.reg.r[rs] as i32) >= 0 {
                            update_pc_next32_with_delayed_imm!(ms, ms.reg.pc + (sign_ext16!(imm) << 2) + 4 );
                        }else{
                            update_pc_next32!(ms);
                        }
                    }
                }
                0x08 => // tgei (trap if greater or equal immediate)
                {
                    if ms.emu.debug { info!("tgei {}, 0x{:>x}", mips::REGSTR[rs], imm); }
                    if (ms.reg.r[rs] as i32) >= (sign_ext16!(imm) as i32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x09 => // tgeiu (trap if greater or equal immediate unsigned)
                {
                    if ms.emu.debug { info!("tgeiu {}, 0x{:>x}", mips::REGSTR[rs], imm); }
                    if (ms.reg.r[rs] as u32) >= (sign_ext16!(imm) as u32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x0a => // tlti (trap if less than immediate)
                {
                    if ms.emu.debug { info!("tlti {}, 0x{:>x}", mips::REGSTR[rs], imm); }
                    if (ms.reg.r[rs] as i32) < (sign_ext16!(imm) as i32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x0b => // tltiu (trap if less than immediate unsigned)
                {
                    if ms.emu.debug { info!("tltiu {}, 0x{:>x}", mips::REGSTR[rs], imm); }
                    if (ms.reg.r[rs] as u32) < (sign_ext16!(imm) as u32) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x0c => // teqi
                {
                    if ms.emu.debug { info!("teqi {}, 0x{:>x}", mips::REGSTR[rs], imm); }
                    if ms.reg.r[rs] == sign_ext16!(imm) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                0x0e => // tnei
                {
                    if ms.emu.debug { info!("tnei {}, 0x{:>x}", mips::REGSTR[rs], imm); }
                    if ms.reg.r[rs] != sign_ext16!(imm) {
                        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_TRAP, 0);
                    }else{
                        update_pc_next32!(ms);
                    }
                }
                _ =>
                {
                    unknown_instruction!(inst,"op=0x01");
                }
            }
            return true;
        }
        MIPS32_OP_LB => // lb
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lb {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_byte(ms, utmp) {
                Ok(data) => { ms.reg.r[rt] = ((data as i8) as i32) as u32; update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_LBU => // lbu
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lbu {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_byte(ms, utmp) {
                Ok(data) => { ms.reg.r[rt] = data & 0xff; update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_LH => // lh
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lh {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_halfword(ms, utmp) {
                Ok(data) => { ms.reg.r[rt] = ((data as i16) as i32) as u32; update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_LHU => // lhu
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lhu {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_halfword(ms, utmp) {
                Ok(data) => { ms.reg.r[rt] = data & 0xffff; update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_LW => // lw
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lw {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_word(ms, utmp) {
                Ok(data) => { ms.reg.r[rt] = data; update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_SB => // sb
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("sb {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::store_byte(ms, utmp, ms.reg.r[rt] as u8) {
                Ok(()) => { update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_SH => // sh
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("sh {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::store_halfword(ms, utmp, 0xffff & ms.reg.r[rt]) {
                Ok(()) => { update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_SW => // sw
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("sw {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::store_word(ms, utmp, ms.reg.r[rt]) {
                Ok(()) => { update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_LWL => // lwl
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lwl {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }

            match mem::load_word(ms, utmp & (!(0x3 as u32))) {
                Ok(data) => 
                { 
                    match utmp & 3 {
                        // TODO: 32-bit big-endian specific implementation
                        0 => {ms.reg.r[rt] = data; }
                        1 => {ms.reg.r[rt] = (ms.reg.r[rt] & 0x000000ff) | (data<< 8); }
                        2 => {ms.reg.r[rt] = (ms.reg.r[rt] & 0x0000ffff) | (data<<16); }
                        3 => {ms.reg.r[rt] = (ms.reg.r[rt] & 0x00ffffff) | (data<<24); }
                        _ => {()}
                    }
                    update_pc_next32!(ms);
                }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp & (!(0x3 as u32))); }
            }
            return true;
        }
        MIPS32_OP_LWR => // lwr
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("lwr {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_word(ms, utmp & (!(0x3 as u32))) {
                Ok(data) => 
                { 
                    match utmp & 3 {
                        // TODO: 32-bit big-endian specific implementation
                        0 => {ms.reg.r[rt] = (ms.reg.r[rt] & 0xffffff00) | ((data>>24)&0x000000ff); }
                        1 => {ms.reg.r[rt] = (ms.reg.r[rt] & 0xffff0000) | ((data>>16)&0x0000ffff); }
                        2 => {ms.reg.r[rt] = (ms.reg.r[rt] & 0xff000000) | ((data>> 8)&0x00ffffff); }
                        3 => {ms.reg.r[rt] = data; }
                        _ => {()}
                    }
                    update_pc_next32!(ms);
                }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp & (!(0x3 as u32))); }
            }
            return true;
        }
        MIPS32_OP_SWL => //swl
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("swl {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_word(ms, utmp & (!(0x3 as u32))) {
                Ok(data) => 
                {
                    match utmp & 3 {
                        // TODO: 32-bit big-endian specific implementation
                        0 => {loaddata = ms.reg.r[rt]; }
                        1 => {loaddata = (data & 0xff000000) | ((ms.reg.r[rt]>> 8)&0x00ffffff); }
                        2 => {loaddata = (data & 0xffff0000) | ((ms.reg.r[rt]>>16)&0x0000ffff); }
                        3 => {loaddata = (data & 0xffffff00) | ((ms.reg.r[rt]>>24)&0x000000ff); }
                        _ => {loaddata = 0; }
                    }
                }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp & (!(0x3 as u32))); return true; }
            }
            match mem::store_word(ms, utmp & (!(0x3 as u32)), loaddata) {
                Ok(()) => { update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp & (!(0x3 as u32))); }
            }
            return true;
        }
        MIPS32_OP_SWR => // swr
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("swr {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            match mem::load_word(ms, utmp & (!(0x3 as u32))) {
                Ok(data) => 
                {
                    match utmp & 3 {
                        // TODO: 32-bit big-endian specific implementation
                        0 => {loaddata = (data & 0x00ffffff) | (ms.reg.r[rt]<<24); }
                        1 => {loaddata = (data & 0x0000ffff) | (ms.reg.r[rt]<<16); }
                        2 => {loaddata = (data & 0x000000ff) | (ms.reg.r[rt]<< 8); }
                        3 => {loaddata = ms.reg.r[rt]; }
                        _ => {loaddata = 0; }
                    }
                }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp & (!(0x3 as u32))); return true; }
            }
            match mem::store_word(ms, utmp & (!(0x3 as u32)), loaddata) {
                Ok(()) => { update_pc_next32!(ms); }
                Err(ecode)  => { exception::prepare_exception(ms, ecode, utmp & (!(0x3 as u32))); }
            }
            return true;
        }
        MIPS32_OP_SC => // sc
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("sc {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            if ms.reg.ll_sc {
                match mem::store_word(ms, utmp, ms.reg.r[rt]) {
                    Ok(()) => 
                    {
                        ms.reg.r[rt] = 1;
                        update_pc_next32!(ms); 
                    }
                    Err(ecode)  => 
                    {
                        exception::prepare_exception(ms, ecode, utmp); 
                        // TODO: no write to rt for retrying this instruction after exception handling
                    }
                }
            }else{
                ms.reg.r[rt] = 0;
                update_pc_next32!(ms);
            }
            return true;
        }
        MIPS32_OP_LL => // ll
        {
            utmp = ms.reg.r[rs] + sign_ext16!(imm);
            if ms.emu.debug { info!("ll {}, 0x{:>x}({}) (=0x{:>x}) ", mips::REGSTR[rt], imm, mips::REGSTR[rs], utmp); }
            ms.reg.ll_sc = true;
            match mem::load_word(ms, utmp) {
                Ok(data) => { ms.reg.r[rt] = data; update_pc_next32!(ms); }
                Err(ecode) => { exception::prepare_exception(ms, ecode, utmp); }
            }
            return true;
        }
        MIPS32_OP_LUI => // lui
        {
            if ms.emu.debug { info!("lui {}, 0x{:>x}", mips::REGSTR[rt], imm); }
            ms.reg.r[rt] = zero_ext16!(imm) << 16;
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_ORI => // ori
        {
            if ms.emu.debug { info!("ori {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }
            ms.reg.r[rt] = ms.reg.r[rs] | zero_ext16!(imm);
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_XORI => // xori
        {
            if ms.emu.debug { info!("xori {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }
            ms.reg.r[rt] = ms.reg.r[rs] ^ zero_ext16!(imm);
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_SLTI => // slti
        {
            if ms.emu.debug { info!("slti {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }
            ms.reg.r[rt] = if (ms.reg.r[rs] as i32) < (sign_ext16!(imm) as i32) { 1 }else{ 0 };
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_SLTIU => // sltiu
        {
            if ms.emu.debug { info!("sltiu {}, {}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], imm); }
            ms.reg.r[rt] = if (ms.reg.r[rs] as u32) < (sign_ext16!(imm) as u32) { 1 }else{ 0 };
            update_pc_next32!(ms);
            return true;
        }
        MIPS32_OP_SPECIAL2 => // SPECIAL2
        {
            if funct == 0x02 && shamt == 0x00 {
                if ms.emu.debug { info!("mul {}, {}, {}", mips::REGSTR[rd], mips::REGSTR[rs], mips::REGSTR[rt]); }
                ms.reg.r[rd] = ((ms.reg.r[rs] as i32) * (ms.reg.r[rt] as i32)) as u32;
                update_pc_next32!(ms);
            }else if imm == 0x00 {
                if ms.emu.debug { info!("madd {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                let hilo : i64 = ((((ms.reg.hi as u64)<<16)<<16) + (ms.reg.lo as u64)) as i64;
                let tmul : i64 = ((ms.reg.r[rs] as i32) as i64) * ((ms.reg.r[rt] as i32) as i64);
                let tmul2: i64 = hilo + tmul;
                ms.reg.hi = ((tmul2>>16)>>16) as u32;
                ms.reg.lo = tmul2 as u32;
                update_pc_next32!(ms);
            }else if imm == 0x01 {
                if ms.emu.debug { info!("maddu {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                let hilo : u64 = (((ms.reg.hi as u64)<<16)<<16) + (ms.reg.lo as u64);
                let tmul0: u64 =  (ms.reg.r[rs] as u64) * (ms.reg.r[rt] as u64);
                let tmul : u64 = hilo + tmul0;
                ms.reg.hi = ((tmul>>16)>>16) as u32;
                ms.reg.lo = tmul as u32;
                update_pc_next32!(ms);
            }else if imm == 0x04 {
                if ms.emu.debug { info!("msub {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                let hilo : i64 = ((((ms.reg.hi as u64)<<16)<<16) + (ms.reg.lo as u64)) as i64;
                let tmul0: i64 = ((ms.reg.r[rs] as i32) as i64) * ((ms.reg.r[rt] as i32) as i64);
                let tmul : i64 = hilo - tmul0;
                ms.reg.hi = ((tmul>>16)>>16) as u32;
                ms.reg.lo = tmul as u32;
                update_pc_next32!(ms);
            }else if imm == 0x05 {
                if ms.emu.debug { info!("msubu {}, {}", mips::REGSTR[rs], mips::REGSTR[rt]); }
                let hilo : u64 = (((ms.reg.hi as u64)<<16)<<16) + (ms.reg.lo as u64);
                let tmul0: u64 =   (ms.reg.r[rs] as u64) * (ms.reg.r[rt] as u64);
                let tmul : u64 = hilo - tmul0;
                ms.reg.hi = ((tmul>>16)>>16) as u32;
                ms.reg.lo = tmul as u32;
                update_pc_next32!(ms);
            }else if funct == 0x21 && shamt == 0x00 {
                if ms.emu.debug { info!("clo {}, {}", mips::REGSTR[rd], mips::REGSTR[rs]); }
                let mut temp:u32 = 32;
                for cnt in (0..32).rev() {
                    if 0 == ((ms.reg.r[rs]) & (1<<cnt)) {
                        temp = 31 - cnt;
                        break;
                    }
                }
                ms.reg.r[rd] = temp;
                update_pc_next32!(ms);
            }else if funct == 0x20 && shamt == 0x00 {
                if ms.emu.debug { info!("clz {}, {}", mips::REGSTR[rd], mips::REGSTR[rs]); }
                let mut temp:u32 = 32;
                for cnt in (0..32).rev() {
                    if 0 != ((ms.reg.r[rs]) & (1<<cnt)) {
                        temp = 31 - cnt;
                        break;
                    }
                }
                ms.reg.r[rd] = temp;
                update_pc_next32!(ms);
            }else{
                unknown_instruction!(inst,"op=0x1c");
            }
            return true;
        }
        MIPS32_OP_SPECIAL3 =>
        {
            match funct {
                0x3b =>
                {
                    if rs != 0 {
                        unknown_instruction!(inst,"op=0x1f");
                    }
                    if ms.emu.debug { info!("rdhwr {}, 0x{:>x}, 0x{:>x}", mips::REGSTR[rt], rd, shamt&0x7); }

                    match rd {
                        0 => /* read CPU ID num */
                        {
                            if mode_is_user!(c0_val!(ms.reg,cp0def::C0_STATUS)) && 0==(c0_val!(ms.reg,cp0def::C0_HWRENA) & (1<<cp0def::C0_HWRENA_BIT_CPUNUM)) {
                                exception::prepare_exception(ms, cp0def::EXCEPT_CODE_RESERVED_INSTRUCTION, 0);
                            }else{
                                ms.reg.r[rt] = 0;
                                update_pc_next32!(ms);
                            }
                        }
                        1 => /* Address step size to be used with the SYNCI instruction, or zero if no caches need be synchronized. */
                        {
                            if mode_is_user!(c0_val!(ms.reg,cp0def::C0_STATUS)) && 0==(c0_val!(ms.reg,cp0def::C0_HWRENA) & (1<<cp0def::C0_HWRENA_BIT_SYNCISTEP)) {
                                exception::prepare_exception(ms, cp0def::EXCEPT_CODE_RESERVED_INSTRUCTION, 0);
                            }else{
                                ms.reg.r[rt] = 0;
                                update_pc_next32!(ms);
                            }
                        }
                        2 => /* High-resolution cycle counter. This register provides read access to the coprocessor 0 Count Register. */
                        {
                            if mode_is_user!(c0_val!(ms.reg,cp0def::C0_STATUS)) && 0==(c0_val!(ms.reg,cp0def::C0_HWRENA) & (1<<cp0def::C0_HWRENA_BIT_CC)) {
                                exception::prepare_exception(ms, cp0def::EXCEPT_CODE_RESERVED_INSTRUCTION, 0);
                            }else{
                                ms.reg.r[rt] = cp0::load_counter_precise(ms);
                                update_pc_next32!(ms);
                            }
                        }
                        3 => /* Resolution of the CC register (CC register increments every second CPU cycle) */
                        {
                            if mode_is_user!(c0_val!(ms.reg,cp0def::C0_STATUS)) && 0==(c0_val!(ms.reg,cp0def::C0_HWRENA) & (1<<cp0def::C0_HWRENA_BIT_CCRES)) {
                                exception::prepare_exception(ms, cp0def::EXCEPT_CODE_RESERVED_INSTRUCTION, 0);
                            }else{
                                ms.reg.r[rt] = config::CPU_FREQ_COUNT_RESOLUTION;
                                update_pc_next32!(ms);
                            }
                        }
                        29 => /* read C0_USERLOCAL Register */
                        {
                            if mode_is_user!(c0_val!(ms.reg,cp0def::C0_STATUS)) && 0==(c0_val!(ms.reg,cp0def::C0_HWRENA) & (1<<cp0def::C0_HWRENA_BIT_UL)) {
                                exception::prepare_exception(ms, cp0def::EXCEPT_CODE_RESERVED_INSTRUCTION, 0);
                            }else{
                                ms.reg.r[rt] = c0_val!(ms.reg, cp0def::C0_USERLOCAL);
                                update_pc_next32!(ms);
                            }
                        }
                        4 =>
                        {
                            ms.reg.r[rt] = 0; update_pc_next32!(ms); /* Performance Counter Pair */
                        }
                        5 => 
                        {
                            ms.reg.r[rt] = 0; update_pc_next32!(ms); /* support for the Release 6 Paired LL/SC family of instructions */
                        }
                        _ =>
                        {
                            exception::prepare_exception(ms, cp0def::EXCEPT_CODE_RESERVED_INSTRUCTION, 0);
                        }
                    }
                }
                0x00 =>
                {
                    if ms.emu.debug { info!("ext {}, {}, 0x{:>x}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], shamt, rd_u32+1); }
                    ms.reg.r[rt] = ms.reg.r[rs] >> shamt;
                    if rd < 31 {
                        ms.reg.r[rt] &= (2<<rd_u32)-1;
                    }
                    update_pc_next32!(ms);
                }
                0x04 =>
                {
                    if ms.emu.debug { info!("ins {}, {}, 0x{:>x}, 0x{:>x}", mips::REGSTR[rt], mips::REGSTR[rs], shamt, rd_u32+1-shamt); }
                    utmp = (1<<((rd_u32+1)-shamt)) -1;
                    utmp = utmp << shamt;
                    if shamt==0 && rd==31 {
                        ms.reg.r[rt] = ms.reg.r[rs];
                    }else{
                        ms.reg.r[rt] = (ms.reg.r[rt] & (!utmp)) | ((ms.reg.r[rs] << shamt) & utmp);
                    }
                    update_pc_next32!(ms);
                }
                0x20 =>
                {
                    if shamt == 0x10 { // seb
                        if ms.emu.debug { info!("seb {}, {}", mips::REGSTR[rd], mips::REGSTR[rt]); }
                        ms.reg.r[rd] = (((ms.reg.r[rt] as u8) as i8) as i32) as u32;
                    }else if shamt == 0x18 { // seh
                        if ms.emu.debug { info!("seh {}, {}", mips::REGSTR[rd], mips::REGSTR[rt]); }
                        ms.reg.r[rd] = (((ms.reg.r[rt] as u16) as i16) as i32) as u32;
                    }else if shamt == 0x02 { // wsbh
                        if ms.emu.debug { info!("wsbh {}, {}", mips::REGSTR[rd], mips::REGSTR[rt]); }
                        ms.reg.r[rd] = ((ms.reg.r[rt] & 0xff00ff00)>>8) | ((ms.reg.r[rt] & 0x00ff00ff)<<8);
                    }else{
                        unknown_instruction!(inst,"op=0x1f");
                    }
                    update_pc_next32!(ms);
                }
                _ =>
                {
                    unknown_instruction!(inst,"op=0x1f");
                }
            }
            return true;
        }
        MIPS32_OP_COP0 => // COP0
        {
            // Availability of CP0 is checked.
            if mode_is_user!( c0_val!(ms.reg, cp0def::C0_STATUS) ) && 0 == (c0_val!(ms.reg, cp0def::C0_STATUS)&(1<<cp0def::C0_STATUS_BIT_CU0)) {
                exception::prepare_exception(ms, cp0def::EXCEPT_CODE_COPROCESSOR_UNAVAIL, 0);
                return true;
            }

            match rs {
                0x04 =>
                {
                    if ms.emu.debug { info!("mtc0 {}, {}, {}", mips::REGSTR[rt], rd, imm & ((1<<mips::CP_SEL_BITS)-1) ); }
                    cp0::store(ms, ( rd as u32, imm & ((1<<mips::CP_SEL_BITS)-1) ), ms.reg.r[rt]);
                    update_pc_next32!(ms);
                }
                0x00 =>
                {
                    if ms.emu.debug { info!("mfc0 {}, {}, {}", mips::REGSTR[rt], rd, imm & ((1<<mips::CP_SEL_BITS)-1) ); }
                    ms.reg.r[rt] = cp0::load(ms, (rd as u32, imm & ((1<<mips::CP_SEL_BITS)-1)));
                    update_pc_next32!(ms);
                }
                0x0b =>
                {
                    if imm == 0x6000 { // di
                        if ms.emu.debug { info!("di {}", mips::REGSTR[rt]); }
                        utmp = cp0::load(ms, cp0def::C0_STATUS);
                        ms.reg.r[rt] = utmp;
                        utmp &= !(1<<cp0def::C0_STATUS_BIT_IE);
                        cp0::store(ms, cp0def::C0_STATUS, utmp);
                        update_pc_next32!(ms);
                    }else if imm == 0x6020 { // ei
                        if ms.emu.debug { info!("ei {}", mips::REGSTR[rt]); }
                        utmp = cp0::load(ms, cp0def::C0_STATUS);
                        ms.reg.r[rt] = utmp;
                        utmp |= 1<<cp0def::C0_STATUS_BIT_IE;
                        cp0::store(ms, cp0def::C0_STATUS, utmp);
                        update_pc_next32!(ms);
                    }
                }
                0x10 =>
                {
                    if inst == 0x42000002 { // tlbwi
                        if ms.emu.debug { info!("tlbwi"); }
                        tlb::write_with_index(ms);
                        update_pc_next32!(ms);
                    }else if inst == 0x42000006 {  // tlbwr
                        if ms.emu.debug { info!("tlbwr"); }
                        tlb::write_with_random(ms);
                        update_pc_next32!(ms);
                    }else if inst == 0x42000008 {  // tlbp
                        if ms.emu.debug { info!("tlbp"); }
                        tlb::probe(ms);
                        update_pc_next32!(ms);
                    }else if inst == 0x42000018 { // eret
                        if ms.emu.debug { info!("eret"); }
                        if 0 != (c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_ERL)) {
                            ms.reg.pc = c0_val!(ms.reg, cp0def::C0_ERROREPC);
                            c0_val!(ms.reg,cp0def::C0_STATUS) &= !(1<<cp0def::C0_STATUS_BIT_ERL);
                        }else{
                            ms.reg.pc = c0_val!(ms.reg, cp0def::C0_EPC);
                            c0_val!(ms.reg,cp0def::C0_STATUS) &= !(1<<cp0def::C0_STATUS_BIT_EXL);
                        }
                        ms.reg.ll_sc = false;
                        // TODO: processing of SRSCTL etc
                    }else if inst == 0x42000020 {
                        if ms.emu.debug { info!("wait"); }

                        if cfg!(target_family = "wasm") {
                            ms.sleep_req = true;
                        }else{
                            thread::sleep( Duration::new(0, config::SYSTEM_TIMER_INTERVAL_IN_USEC * 1000/4));
                        }

                        update_pc_next32!(ms);
                    }else{
                        unknown_instruction!(inst,"op=0x10, coprocessor instruction");
                    }
                }
                _ =>
                {
                    unknown_instruction!(inst,"op=0x10, coprocessor instruction");
                }
            }
        }
        MIPS32_OP_CACHE => // cache
        {
            if ms.emu.debug { info!("cache 0x{:>x}, 0x{:>x}({})", rt, imm, mips::REGSTR[rs]); }
            update_pc_next32!(ms);
        }
        MIPS32_OP_PREF => // pref
        {
            if ms.emu.debug { info!("pref 0x{:>x}, 0x{:>x}({})", rt, imm, mips::REGSTR[rs]); }
            update_pc_next32!(ms);
        }
        _ =>
        {
            unknown_instruction!(inst,"unhandled op");
        }
    }

    true
}
