use crate::procstate::MachineState;
use crate::cp0def;
use crate::mips;
use crate::mode_is_in_exception;
use crate::c0_val;
use crate::except_vect_all_other;
use crate::except_vect_cache_err;
use crate::except_vect_int;
use crate::except_vect_tlb_refill;


/*
Preparation for entering exception excepting software and hardware interrupts and syscalls.

This function updates PC, C0_EPC, C0_STATUS, and C0_CAUSE.
It also updates related coprocessor registers.

* Arguments
ecode:
 exception code for C0_CAUSE_EXC such as EXCEPT_CODE_TLB_LOAD.

option:
 coprocessor number when ecode is EXCEPT_CODE_COPROCESSOR_UNAVAIL
 BADVADDR           when ecode is EXCEPT_CODE_TLB_LOAD
 BADVADDR           when ecode is EXCEPT_CODE_TLB_STORE
 BADVADDR           when ecode is EXCEPT_CODE_ADDR_ERR_LOAD
 BADVADDR           when ecode is EXCEPT_CODE_ADDR_ERR_STORE
*/
pub fn prepare_exception(ms: &mut MachineState, ecode : u32, option : u32){

    let prev_mode_is_exl:bool = mode_is_in_exception!(c0_val!(ms.reg,cp0def::C0_STATUS));

    c0_val!(ms.reg,cp0def::C0_CAUSE) &= !cp0def::C0_CAUSE_EXCCODE_MASK;
    c0_val!(ms.reg,cp0def::C0_CAUSE) |= (ecode << cp0def::C0_CAUSE_BIT_EXCCODE) & cp0def::C0_CAUSE_EXCCODE_MASK;
    c0_val!(ms.reg,cp0def::C0_CAUSE) &= !cp0def::C0_CAUSE_CE_MASK;

    if ecode == cp0def::EXCEPT_CODE_COPROCESSOR_UNAVAIL {
        c0_val!(ms.reg,cp0def::C0_CAUSE) |= (option << cp0def::C0_CAUSE_BIT_CE) & cp0def::C0_CAUSE_CE_MASK;
    }

    if prev_mode_is_exl {
        if ms.reg.delay_en {
            c0_val!(ms.reg, cp0def::C0_ERROREPC)= ms.reg.pc_prev_jump;
        }else{
            c0_val!(ms.reg, cp0def::C0_ERROREPC)= ms.reg.pc;
        }
    }else{
        if ms.reg.delay_en {
            c0_val!(ms.reg, cp0def::C0_EPC)     = ms.reg.pc_prev_jump;
            c0_val!(ms.reg, cp0def::C0_CAUSE)  |=  1<<cp0def::C0_CAUSE_BIT_BD;
        }else{
            c0_val!(ms.reg, cp0def::C0_EPC)     =  ms.reg.pc;
            c0_val!(ms.reg, cp0def::C0_CAUSE)  &= !(1<<cp0def::C0_CAUSE_BIT_BD);
        }
    }

    ms.reg.delay_en = false;

    match ecode {
        cp0def::EXCEPT_CODE_CACHE_ERROR =>
        {
            c0_val!(ms.reg,cp0def::C0_STATUS) |= 1<<cp0def::C0_STATUS_BIT_ERL; // error level
            ms.reg.pc = except_vect_cache_err!( c0_val!(ms.reg,cp0def::C0_EBASE), c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_BEV) );
        }
        cp0def::EXCEPT_CODE_TLB_REFILL_LOAD | cp0def::EXCEPT_CODE_TLB_REFILL_STORE =>
        {
            /*
            See:
            Figure 6.6 in page 141 of
            "MIPS32 74K Processor Core Family Software User’s Manual," Revision 01.05, March 30, 2011.
            */
            c0_val!(ms.reg,cp0def::C0_CAUSE) &= !cp0def::C0_CAUSE_EXCCODE_MASK;
            c0_val!(ms.reg,cp0def::C0_CAUSE) |= 
                (if ecode == cp0def::EXCEPT_CODE_TLB_REFILL_LOAD {cp0def::EXCEPT_CODE_TLB_LOAD}else{cp0def::EXCEPT_CODE_TLB_STORE}) << cp0def::C0_CAUSE_BIT_EXCCODE;

            c0_val!(ms.reg,cp0def::C0_STATUS)  |= 1<<cp0def::C0_STATUS_BIT_EXL; // exception level
            c0_val!(ms.reg,cp0def::C0_ENTRYHI) &= !cp0def::C0_ENTRYHI_VPN2_MASK;
            c0_val!(ms.reg,cp0def::C0_ENTRYHI) |= option & cp0def::C0_ENTRYHI_VPN2_MASK;
            c0_val!(ms.reg,cp0def::C0_CONTEXT) &= 0xff80000f; /* TODO: position is variable. check is necessary */
            c0_val!(ms.reg,cp0def::C0_CONTEXT) |= (option & cp0def::C0_ENTRYHI_VPN2_MASK) >> 9; /* TODO: position is variable. check is necessary */
            c0_val!(ms.reg,cp0def::C0_BADVADDR) = option;
            ms.reg.pc = except_vect_tlb_refill!( c0_val!(ms.reg,cp0def::C0_EBASE), c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_BEV), prev_mode_is_exl );
        }
        cp0def::EXCEPT_CODE_TLB_LOAD | cp0def::EXCEPT_CODE_TLB_STORE | cp0def::EXCEPT_CODE_MOD =>
        {
            /*
            See:
            Figure 6.6 in page 141 of
            "MIPS32 74K Processor Core Family Software User’s Manual," Revision 01.05, March 30, 2011, page 175-176.
            */
            c0_val!(ms.reg,cp0def::C0_STATUS)  |= 1<<cp0def::C0_STATUS_BIT_EXL; // exception level
            c0_val!(ms.reg,cp0def::C0_ENTRYHI) &= !cp0def::C0_ENTRYHI_VPN2_MASK;
            c0_val!(ms.reg,cp0def::C0_ENTRYHI) |= option & cp0def::C0_ENTRYHI_VPN2_MASK;
            c0_val!(ms.reg,cp0def::C0_CONTEXT) &= 0xff80000f; /* TODO: position is variable. check is necessary */
            c0_val!(ms.reg,cp0def::C0_CONTEXT) |= (option & cp0def::C0_ENTRYHI_VPN2_MASK) >> 9; /* TODO: position is variable. check is necessary */
            c0_val!(ms.reg,cp0def::C0_BADVADDR) = option;
            ms.reg.pc = except_vect_all_other!( c0_val!(ms.reg,cp0def::C0_EBASE), c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_BEV) );
        }
        cp0def::EXCEPT_CODE_ADDR_ERR_LOAD | cp0def::EXCEPT_CODE_ADDR_ERR_STORE =>
        {
            c0_val!(ms.reg,cp0def::C0_BADVADDR) = option;
            // go through
            c0_val!(ms.reg,cp0def::C0_STATUS) |= 1<<cp0def::C0_STATUS_BIT_EXL; // exception level
            ms.reg.pc = except_vect_all_other!( c0_val!(ms.reg,cp0def::C0_EBASE), c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_BEV) );
        }
        _ =>
        {
            c0_val!(ms.reg,cp0def::C0_STATUS) |= 1<<cp0def::C0_STATUS_BIT_EXL; // exception level
            ms.reg.pc = except_vect_all_other!( c0_val!(ms.reg,cp0def::C0_EBASE), c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_BEV) );
        }
    }
}

pub fn prepare_interrupt(ms: &mut MachineState, icode : u32){

    let prev_mode_is_exl:bool = mode_is_in_exception!(c0_val!(ms.reg,cp0def::C0_STATUS));

    c0_val!(ms.reg,cp0def::C0_CAUSE) &= !cp0def::C0_CAUSE_EXCCODE_MASK;
    c0_val!(ms.reg,cp0def::C0_CAUSE) |= cp0def::EXCEPT_CODE_INTERRUPT << cp0def::C0_CAUSE_BIT_EXCCODE;

    c0_val!(ms.reg,cp0def::C0_CAUSE) &= !cp0def::C0_CAUSE_IP_MASK;
    c0_val!(ms.reg,cp0def::C0_CAUSE) |= (icode << cp0def::C0_CAUSE_BIT_IP) & cp0def::C0_CAUSE_IP_MASK;

    if ! prev_mode_is_exl {
        if ms.reg.delay_en {
            c0_val!(ms.reg,cp0def::C0_EPC)    =  ms.reg.pc_prev_jump;
            c0_val!(ms.reg,cp0def::C0_CAUSE) |=  1<<cp0def::C0_CAUSE_BIT_BD;
        }else{
            c0_val!(ms.reg,cp0def::C0_EPC)    =  ms.reg.pc;
            c0_val!(ms.reg,cp0def::C0_CAUSE) &= !(1<<cp0def::C0_CAUSE_BIT_BD);
        }
    }

    ms.reg.delay_en = false;

    c0_val!(ms.reg,cp0def::C0_STATUS) |= 1<<cp0def::C0_STATUS_BIT_EXL; // exception level
    ms.reg.pc = except_vect_int!( c0_val!(ms.reg,cp0def::C0_EBASE), c0_val!(ms.reg,cp0def::C0_STATUS) & (1<<cp0def::C0_STATUS_BIT_BEV), c0_val!(ms.reg,cp0def::C0_CAUSE) & (1<<cp0def::C0_CAUSE_BIT_IV) );
}