use crate::procstate::MachineState;
use crate::{config, cp0def};
use crate::mips;
use crate::c0_val;
use log::{error,info};

macro_rules! store_masked_val
{ ( $val:expr, $reg_set:expr ) => ( ($val & $reg_set.mask_w) | $reg_set.const_val ) }
macro_rules! load_masked_val
{ ( $val:expr, $reg_set:expr ) => ( ($val & $reg_set.mask_r) | $reg_set.const_val ) }


pub fn store_counter(ms : &mut MachineState, val : u32){
    let diff : u64 = (val as u64) / ((config::FREQ_CPU/(config::CPU_FREQ_COUNT_RESOLUTION*1000*1000)) as u64);

    ms.reg.c0_count_basetime = ms.reg.c0_count_currenttime - diff;
}

pub fn load_counter_precise(ms : &mut MachineState) -> u32 {
    return load_counter(ms);
}

pub fn load_counter(ms : &mut MachineState) -> u32 {
    let counter_t  :u64 = ms.reg.c0_count_currenttime - ms.reg.c0_count_basetime;
    let counter_cyc:u64 = counter_t * ((config::FREQ_CPU/(config::CPU_FREQ_COUNT_RESOLUTION*1000*1000)) as u64);
    let counter_cur:u64 = counter_cyc + (ms.emu.nexec_insts - ms.reg.c0_count_ninst_in_ctime);
    return counter_cur as u32;
}

pub fn load_counter_long(ms : &mut MachineState) -> u64 {
    let counter_t  :u64 = ms.reg.c0_count_currenttime - ms.reg.c0_count_basetime;
    let counter_cyc:u64 = counter_t * ((config::FREQ_CPU/(config::CPU_FREQ_COUNT_RESOLUTION*1000*1000)) as u64);
    let counter_cur:u64 = counter_cyc + (ms.emu.nexec_insts - ms.reg.c0_count_ninst_in_ctime);
    return counter_cur;
}

pub fn store(ms : &mut MachineState, (reg,sel) : (u32,u32), val : u32){
    let rs = (reg & ((1<<mips::CP_REG_BITS)-1), sel & ((1<<mips::CP_SEL_BITS)-1));

    match rs {
        cp0def::C0_STATUS   => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_STATUS_SETTING  ); }
        cp0def::C0_CAUSE    => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_CAUSE_SETTING   ); }

        cp0def::C0_ENTRYHI  => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_ENTRYHI_SETTING ); }
        cp0def::C0_ENTRYLO0 => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_ENTRYLO0_SETTING); }
        cp0def::C0_ENTRYLO1 => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_ENTRYLO1_SETTING); }
        cp0def::C0_RANDOM   => { /* Do nothing because read-only */ }
        cp0def::C0_INDEX    => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_INDEX_SETTING   ); }
        cp0def::C0_PAGEMASK => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_PAGEMASK_SETTING); }
        cp0def::C0_WIRED    => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_WIRED_SETTING   ); }
        cp0def::C0_BADVADDR => { /* Do nothing because read-only */ }

        cp0def::C0_HWRENA   => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_HWRENA_SETTING  ); }
        cp0def::C0_EBASE    => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_EBASE_SETTING   ); }
        cp0def::C0_CONFIG   => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_CONFIG_SETTING  ); }
        cp0def::C0_CONFIG2  => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_CONFIG2_SETTING ); }

        cp0def::C0_INTCTL   => { c0_val!(ms.reg,rs) = store_masked_val!(val, cp0def::C0_INTCTL_SETTING  ); }

        cp0def::C0_COUNT    => { store_counter(ms, val); }
        cp0def::C0_COMPARE  => {
            let long_count:u64 = load_counter_long(ms);
            if val <= (long_count as u32) {
                ms.reg.c0_compare_long = (long_count + 0x100000000) & 0xffffffff00000000;
                if ms.reg.c0_compare_long < long_count {
                    error!("Wrap around occurs ---------------------------------");
                }
            }else{
                ms.reg.c0_compare_long = long_count & 0xffffffff00000000;
            }
            ms.reg.c0_compare_long += val as u64;
            c0_val!(ms.reg, cp0def::C0_COMPARE) = val;
            c0_val!(ms.reg, cp0def::C0_CAUSE) &= !((1<<cp0def::C0_INTCTL_TIMER_INT_IPNUM)<<cp0def::C0_CAUSE_BIT_IP);
            c0_val!(ms.reg, cp0def::C0_CAUSE) &= ! (1<<cp0def::C0_CAUSE_BIT_TI);
        }
        cp0def::C0_EPC      => { c0_val!(ms.reg,rs) = val; }
        cp0def::C0_CONTEXT  => { c0_val!(ms.reg,rs) = val; }
        _ => { c0_val!(ms.reg,rs) = val; info!("Write CP0(pc: 0x{:>x}, reg: {}, sel: {}, val: 0x{:>x})\r", ms.reg.pc, reg, sel, val) }
    }

}

pub fn load(ms : &mut MachineState, (reg,sel) : (u32,u32) ) -> u32 {
    let rs:(u32,u32) = (reg & ((1<<mips::CP_REG_BITS)-1), sel & ((1<<mips::CP_SEL_BITS)-1));

    match rs {
        cp0def::C0_STATUS    =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_STATUS_SETTING   ); }
        cp0def::C0_CAUSE     =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_CAUSE_SETTING    ); }

        cp0def::C0_ENTRYHI   =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_ENTRYHI_SETTING  ); }
        cp0def::C0_ENTRYLO0  =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_ENTRYLO0_SETTING ); }
        cp0def::C0_ENTRYLO1  =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_ENTRYLO1_SETTING ); }
        cp0def::C0_INDEX     =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_INDEX_SETTING    ); }
        cp0def::C0_PAGEMASK  =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_PAGEMASK_SETTING ); }
        cp0def::C0_WIRED     =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_WIRED_SETTING    ); }
        cp0def::C0_RANDOM    =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_RANDOM_SETTING   ); }

        cp0def::C0_HWRENA    =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_HWRENA_SETTING   ); }
        cp0def::C0_EBASE     =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_EBASE_SETTING    ); }
        cp0def::C0_CONFIG    =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_CONFIG_SETTING   ); }
        cp0def::C0_CONFIG1   =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_CONFIG1_SETTING  ); }
        cp0def::C0_CONFIG2   =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_CONFIG2_SETTING  ); }
        cp0def::C0_CONFIG3   =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_CONFIG3_SETTING  ); }
        cp0def::C0_PRID      =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_PRID_SETTING     ); }

        cp0def::C0_INTCTL    =>{ return load_masked_val!(c0_val!(ms.reg,rs), cp0def::C0_INTCTL_SETTING ); }

        cp0def::C0_COUNT     =>{ return load_counter(ms); }
        cp0def::C0_COMPARE | cp0def::C0_EPC | cp0def::C0_CONTEXT | cp0def::C0_BADVADDR  =>{ return c0_val!(ms.reg,rs); }
        _                   => { info!("Read CP0(pc: 0x{:>x}, reg: {}, sel: {}, val: 0x{:>x})\r", ms.reg.pc, reg, sel, c0_val!(ms.reg,rs)); return c0_val!(ms.reg,rs); }
    }
}
