use crate::procstate::MachineState;
use crate::cp0def;
use crate::config;
use crate::mips;

use crate::c0_val;

#[derive(Copy,Clone)]
pub struct TLBPhyAddr{
    pub field_pfn   : u32,
    pub field_dirty : bool,
    pub field_valid : bool
}

#[derive(Copy,Clone)]
pub struct TLBEntry{
    pub entryhi : u32,
    pub entrylo0: u32,
    pub entrylo1: u32,

    pub field_asid : u32, // located in entryhi[ 7: 0]
    pub field_pmask: u32, // located in pagemask
    pub field_vpn2 : u32, // located in entryhi[31:13]
    pub field_g    : bool,// logical AND of g bits of EntryLo0 and EntryLo1
    pub lo         : [TLBPhyAddr; 2],
}


impl TLBEntry {
    pub fn new() -> Self {
        Self { 
            entryhi : 0,
            entrylo0: 0,
            entrylo1: 0,

            field_asid : 0,  // located in entryhi[ 7: 0]
            field_pmask: 0, // located in pagemask
            field_vpn2 : 0, // located in entryhi[31:13]
            field_g    : false,// logical AND of g bits of EntryLo0 and EntryLo1
            lo         : [TLBPhyAddr { field_pfn: 0, field_dirty: false, field_valid: false}; 2]
        }
    }
}

fn tlb_write(ms: &mut MachineState, index : u32){
    let entryhi  :u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI );
    let entrylo0 :u32 = c0_val!(ms.reg, cp0def::C0_ENTRYLO0);
    let entrylo1 :u32 = c0_val!(ms.reg, cp0def::C0_ENTRYLO1);

    ms.reg.pc_cache.clear();
    ms.reg.dr_cache[0].clear();
    ms.reg.dr_cache[1].clear();
    ms.reg.dw_cache[0].clear();
    ms.reg.dw_cache[1].clear();

    let rawidx : usize = (index & cp0def::C0_INDEX_INDEX_MASK) as usize;
    let idx : usize = if rawidx >= config::NUM_TLB_ENTRY as usize { rawidx % config::NUM_TLB_ENTRY as usize }else{ rawidx };

    ms.tlbcache[((ms.tlb[idx].entryhi >> 12) as usize) & (config::TLB_CACHE_SIZE-1)] = config::TLB_CACHE_SIZE as u8;
    ms.tlbcache[((entryhi             >> 12) as usize) & (config::TLB_CACHE_SIZE-1)] = idx as u8;

    ms.tlb[idx].entryhi  = entryhi;
    ms.tlb[idx].entrylo0 = entrylo0;
    ms.tlb[idx].entrylo1 = entrylo1;

    ms.tlb[idx].field_vpn2 = entryhi & cp0def::C0_ENTRYHI_VPN2_MASK;
    ms.tlb[idx].field_asid = entryhi & cp0def::C0_ENTRYHI_ASID_MASK;
    ms.tlb[idx].field_g    = if 0 != (entrylo0 & entrylo1 & 1) { true }else{ false };
    ms.tlb[idx].field_pmask= c0_val!(ms.reg, cp0def::C0_PAGEMASK);

    ms.tlb[idx].lo[0].field_valid  = if 0 != (entrylo0 & 2) { true }else{ false };
    ms.tlb[idx].lo[0].field_dirty  = if 0 != (entrylo0 & 4) { true }else{ false };
    ms.tlb[idx].lo[0].field_pfn    = (entrylo0<<6) & 0xfffff000;

    ms.tlb[idx].lo[1].field_valid  = if 0 != (entrylo1 & 2) { true }else{ false };
    ms.tlb[idx].lo[1].field_dirty  = if 0 != (entrylo1 & 4) { true }else{ false };
    ms.tlb[idx].lo[1].field_pfn    = (entrylo1<<6) & 0xfffff000;
}

pub fn write_with_index(ms : &mut MachineState){
    let idx : u32 = c0_val!(ms.reg, cp0def::C0_INDEX);

    tlb_write(ms, idx);
}



pub fn write_with_random(ms : &mut MachineState){
    let idx : u32 = c0_val!(ms.reg, cp0def::C0_RANDOM);

    // Relation between C0_WIRED and C0_RANDOM
    // 0 <= C0_WIRED <= C0_RANDOM < NUM_TLB_ENTRY

    c0_val!(ms.reg, cp0def::C0_RANDOM) = 
    if idx  <= (c0_val!(ms.reg, cp0def::C0_WIRED) & cp0def::C0_INDEX_INDEX_MASK) {
        config::NUM_TLB_ENTRY -1
    }else if idx >= config::NUM_TLB_ENTRY {
        config::NUM_TLB_ENTRY -1
    }else{
        idx - 1
    };

    tlb_write(ms, idx);
}

pub fn probe(ms : &mut MachineState){
    let addr : u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI) & cp0def::C0_ENTRYHI_VPN2_MASK;
    let asid : u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI) & cp0def::C0_ENTRYHI_ASID_MASK;

    for i in 0..config::NUM_TLB_ENTRY as usize {
        let addrmask   : u32 = 0xfff | ms.tlb[i].field_pmask;
        let addrmask2  : u32 = (addrmask<<1) | 1;
        let maskedaddr : u32 = addr                 & (!addrmask2);
        let maskedvpn2 : u32 = ms.tlb[i].field_vpn2 & (!addrmask2);

        if (maskedvpn2 == maskedaddr) &&
           ((ms.tlb[i].field_g) || (asid == ms.tlb[i].field_asid)) 
        {
            c0_val!(ms.reg, cp0def::C0_INDEX) = i as u32;
            return ;
        }
    }

    c0_val!(ms.reg, cp0def::C0_INDEX) |= 1<<cp0def::C0_INDEX_BIT_P;
}

/*
TLB lookup:

See Section 5.4 (p.100-104) of
"MIPS32 74K Processor Core Family Software User’s Manual," Revision 01.05, March 30, 2011, page 175-176.
*/
pub fn lookup(ms : &mut MachineState, asid : u32, is_write : bool, addr : u32) -> Result<u32, u32> {

    let default_error : u32 = 
    if is_write { cp0def::EXCEPT_CODE_TLB_REFILL_STORE }else{ cp0def::EXCEPT_CODE_TLB_REFILL_LOAD };

    let idx = ms.tlbcache[ ((addr >> 12) as usize) & (config::TLB_CACHE_SIZE-1) ] as usize;

    if idx < config::NUM_TLB_ENTRY as usize {
        let addrmask  : u32 = 0xfff | ms.tlb[idx].field_pmask;
        let addrmask2 : u32 = (addrmask<<1) | 1;
        let vpnlsb    : u32 = addrmask2 ^ addrmask;

        let maskedaddr: u32 = addr                   & (!addrmask2);
        let maskedvpn2: u32 = ms.tlb[idx].field_vpn2 & (!addrmask2);
        let odd       :usize= if (addr & vpnlsb) != 0 { 1 }else{ 0 };

        if (maskedvpn2 == maskedaddr) && ( ms.tlb[idx].field_g || asid == ms.tlb[idx].field_asid ) {
            if ! ms.tlb[idx].lo[odd].field_valid {
                return Err( if is_write { cp0def::EXCEPT_CODE_TLB_STORE }else{ cp0def::EXCEPT_CODE_TLB_LOAD } );
            }

            if is_write && (!ms.tlb[idx].lo[odd].field_dirty) {
                return Err( cp0def::EXCEPT_CODE_MOD );
            }

            return Ok((ms.tlb[idx].lo[odd].field_pfn) | (addr & addrmask));
        }
    }

    for i  in 0..config::NUM_TLB_ENTRY as usize {
        let addrmask  : u32 = 0xfff | ms.tlb[i].field_pmask;
        let addrmask2 : u32 = (addrmask<<1) | 1;
        let vpnlsb    : u32 = addrmask2 ^ addrmask;

        let maskedaddr: u32 = addr                 & (!addrmask2);
        let maskedvpn2: u32 = ms.tlb[i].field_vpn2 & (!addrmask2);
        let odd       :usize= if (addr & vpnlsb) != 0 { 1 }else{ 0 };

        if maskedvpn2 != maskedaddr { continue; }

        if (!ms.tlb[i].field_g) && (asid != ms.tlb[i].field_asid) {
            //*perror = EXCEPT_CODE_TLB_REFILL;
            continue;
        }

        if ! ms.tlb[i].lo[odd].field_valid {
            return Err( if is_write { cp0def::EXCEPT_CODE_TLB_STORE }else{ cp0def::EXCEPT_CODE_TLB_LOAD } );
        }

        if is_write && (!ms.tlb[i].lo[odd].field_dirty) {
            return Err( cp0def::EXCEPT_CODE_MOD );
        }

        ms.tlbcache[ ((addr >> 12) as usize) & (config::TLB_CACHE_SIZE-1) ] = i as u8;

        return Ok((ms.tlb[i].lo[odd].field_pfn) | (addr & addrmask));
    }

    return Err( default_error );
}
