#![allow(dead_code)]

use crate::config;

pub struct C0RegSetting {
    pub mask_r    : u32, // 1 for variable bits
    pub mask_w    : u32, // 1 for writable bits
    pub init_val  : u32,
    pub const_val : u32
}

// Definitions for C0_CONFIG
pub const C0_CONFIG_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : (1<<19 /*Write control*/) | (1<<18 /*Writable*/) | (3 /*Kseg0 coherency attribute*/), /* 1 for variable bits */
    mask_w   : (1<<19 /*Write control*/) | (1<<18 /*Writable*/) | (3 /*Kseg0 coherency attribute*/), /* 1 for writable bits */
    init_val : (1<<18 /*Writable*/) | (2<<0 /* Kseg0 is uncached */),
    const_val: (1<<31 /*CONFIG1 is available*/) | (1<<10 /*MIPS32R2*/) | (1<<7 /*MMU-type:TLB*/),
};

// Definitions for C0_CONFIG1
const C0_CONFIG1_INIT_VAL : u32 = 
    (1<<31 /*CONFIG2 is available*/) | 
    ((config::NUM_TLB_ENTRY-1)<<25) | 
    (2<<22 /*#sets per way*/) | (4<<19 /*line size*/) | (3<<16 /*#assoc*/) /*L1 I-cache*/ | 
    (2<<13 /*#sets per way*/) | (4<<10 /*line size*/) | (3<< 7 /*#assoc*/) /*L1 D-cache*/ | 
    (0<<6) /*existence of CP2*/ | 
    (0<<5) /*MDMX ASE is not implemented*/ | 
    (1<<4) /*#performance counter*/ | 
    (0<<3) /*#watchpoint registers*/ | 
    (0<<2) /*MIPS16e is not available*/ | 
    (0<<1) /*EJTAG is not available*/ | 
    (0<<0) /*floating point unit is not available*/;

pub const C0_CONFIG1_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0,
    mask_w   : 0,
    init_val : C0_CONFIG1_INIT_VAL,
    const_val: C0_CONFIG1_INIT_VAL,
};


// Definitions for C0_CONFIG2
pub const C0_CONFIG2_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 1<<12, /*L2 bypass*/
    mask_w   : 1<<12, /*L2 bypass*/
    init_val : 1<<31, /*Config3 is available*/
    const_val: 1<<31, /*Config3 is available*/
};

// Definitions for C0_CONFIG3
pub const C0_CONFIG3_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0,
    mask_w   : 0,
    init_val : 1<<13, /*USERLOCAL is implemented*/
    const_val: 1<<13, /*USERLOCAL is implemented*/
};

pub const C0_EBASE_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0x3ffff<<12, /*base address of exception vectors*/
    mask_w   : 0x3ffff<<12, /*base address of exception vectors*/
    init_val : 1<<31,
    const_val: 1<<31,
};

// Definitions for C0_PRID
pub const MIPS_PRID_COMPANY_ID_MIPS_TECHNOLOGY : u32 = 0x01;
pub const MIPS_PRID_PROCESSOR_ID_74K_CORE      : u32 = 0x97;

pub const C0_PRID_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0,
    mask_w   : 0,
    init_val : (MIPS_PRID_COMPANY_ID_MIPS_TECHNOLOGY<<16) | (MIPS_PRID_PROCESSOR_ID_74K_CORE<<8),
    const_val: (MIPS_PRID_COMPANY_ID_MIPS_TECHNOLOGY<<16) | (MIPS_PRID_PROCESSOR_ID_74K_CORE<<8),
};

// Definitions for C0_INTCTL
pub const C0_INTCTL_TIMER_INT_IPNUM : u32 = 7;   /* IP num for Timer int.*/
pub const C0_INTCTL_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0xf<<5, /* spacing between vectored interrupts */
    mask_w   : 0xf<<5, /* spacing between vectored interrupts */
    init_val : (C0_INTCTL_TIMER_INT_IPNUM<<29) | (5<<26 /**/) | (4<<23 /**/),
    const_val: (C0_INTCTL_TIMER_INT_IPNUM<<29) | (5<<26 /**/) | (4<<23 /**/),
};

// Definitions for C0_ENTRYHI
pub const C0_ENTRYHI_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : !0x00001f00,
    mask_w   : !0x00001f00,
    init_val : 0,
    const_val: 0,
};

// Definitions for C0_ENTRYLO0,1
pub const C0_ENTRYHI_BIT_ASID : u32 =  0;
pub const C0_ENTRYHI_BIT_VPN2 : u32 =  13;
pub const C0_ENTRYHI_ASID_MASK: u32 =  0xff;
pub const C0_ENTRYHI_VPN2_MASK: u32 =  !0x1fff;

pub const C0_ENTRYLO0_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : !0xfc000000,
    mask_w   : !0xfc000000,
    init_val : 0,
    const_val: 0,
};
pub const C0_ENTRYLO1_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : !0xfc000000,
    mask_w   : !0xfc000000,
    init_val : 0,
    const_val: 0,
};

// Definitions for C0_INDEX
pub const C0_INDEX_BIT_P      : u32 = 31; /* Probe Failure */
pub const C0_INDEX_INDEX_MASK : u32 = 0x3f; /* mask for TLB index */

pub const C0_INDEX_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0x8000003f,
    mask_w   : 0x8000003f,
    init_val : 0,
    const_val: 0,
};


// Definitions for C0_RANDOM
pub const C0_RANDOM_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0x0000003f,
    mask_w   : 0x00000000,
    init_val : config::NUM_TLB_ENTRY - 1,
    const_val: 0,
};

// Definitions for C0_PAGEMASK
pub const C0_PAGEMASK_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0xffff<<13,
    mask_w   : 0xffff<<13,
    init_val : 0,
    const_val: 0,
};

// Definitions for C0_WIRED
pub const C0_WIRED_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : 0x3f,
    mask_w   : 0x3f,
    init_val : 0,
    const_val: 0,
};

// Definitions for C0_HWRENA
pub const C0_HWRENA_BIT_UL        : u32 =  29; /* enable "rdhwr 29" to read C0_USERLOCAL register in user mode */
pub const C0_HWRENA_BIT_CCRES     : u32 =  3;  /* enable "rdhwr 3" to read resolution of the CC register in user mode */
pub const C0_HWRENA_BIT_CC        : u32 =  2;  /* enable "rdhwr 2" to read high-resolution cycle counter, C0_COUNT, in user mode */
pub const C0_HWRENA_BIT_SYNCISTEP : u32 =  1;  /* enable "rdhwr 1" to read address step size to be used with the SYNCI instruction in user mode */
pub const C0_HWRENA_BIT_CPUNUM    : u32 =  0;  /* enable "rdhwr 0" to read CPU ID number in user mode */

pub const C0_HWRENA_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : (1<<C0_HWRENA_BIT_UL) | (1<<C0_HWRENA_BIT_CCRES) | (1<<C0_HWRENA_BIT_CC) | (1<<C0_HWRENA_BIT_SYNCISTEP) | (1<<C0_HWRENA_BIT_CPUNUM),
    mask_w   : (1<<C0_HWRENA_BIT_UL) | (1<<C0_HWRENA_BIT_CCRES) | (1<<C0_HWRENA_BIT_CC) | (1<<C0_HWRENA_BIT_SYNCISTEP) | (1<<C0_HWRENA_BIT_CPUNUM),
    init_val : 0,
    const_val: 0,
};

/*
Definitions for C0_STATUS

Values are from
"MIPS32 74K Processor Core Family Software User’s Manual," Revision 01.05, March 30, 2011, page 164-166.
*/
pub const C0_STATUS_BIT_IE   : u32 = 0;  /* Interrupt Enable */
pub const C0_STATUS_BIT_EXL  : u32 = 1;  /* Exception Level */
pub const C0_STATUS_BIT_ERL  : u32 = 2;  /* Error Level */
pub const C0_STATUS_BIT_SM   : u32 = 3;  /* Supervisor Mode */
pub const C0_STATUS_BIT_UM   : u32 = 4;  /* User Mode */
pub const C0_STATUS_BIT_IM   : u32 = 8;  /* Interrupt Mask */
pub const C0_STATUS_IM_MASK  : u32 = 0xff<<C0_STATUS_BIT_IM;
pub const C0_STATUS_BIT_CEE  : u32 = 17; /* CorExtend Enable. Enable/disable CorExtend User Defined Instructions */
pub const C0_STATUS_BIT_NMI  : u32 = 19; /* Indicates that the entry through the reset exception vector was due to an NMI */
pub const C0_STATUS_BIT_SR   : u32 = 20; /* Soft Reset */
pub const C0_STATUS_BIT_TS   : u32 = 21; /* TLB Shutdown */
pub const C0_STATUS_BIT_BEV  : u32 = 22; /* Boot Exception Vector. Controls the location of exception vectors */
pub const C0_STATUS_BIT_MX   : u32 = 24; /* MIPS Extension. Enables access to DSP ASE resources */
pub const C0_STATUS_BIT_RE   : u32 = 25; /* Reverse Endian */
pub const C0_STATUS_BIT_FR   : u32 = 26; /* Floating Register */
pub const C0_STATUS_BIT_RP   : u32 = 27; /* Reduced Power */
pub const C0_STATUS_BIT_CU0  : u32 = 28; /* Coprocessor 0 Usable (1: access allowed in user mode) */
pub const C0_STATUS_BIT_CU1  : u32 = 29; /* Coprocessor 1 Usable */
pub const C0_STATUS_BIT_CU2  : u32 = 30; /* Coprocessor 2 Usable */
pub const C0_STATUS_BIT_CU3  : u32 = 31; /* Coprocessor 3 Usable */

//  UM SM
//   0  0 : Kernel
//   0  1 : Supervisor
//   1  0 : User
pub const C0_STATUS_KSU_MASK : u32 =  0x3<<C0_STATUS_BIT_SM;
pub const C0_STATUS_BIT_KSU  : u32 =  C0_STATUS_BIT_SM;


/* 1: writable, 0: read-only */
pub const C0_STATUS_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : !((1<<C0_STATUS_BIT_CU3) | (1<<C0_STATUS_BIT_CU2) | (1<<C0_STATUS_BIT_RE) | (1<<C0_STATUS_BIT_SR) | (1<<23 /*reserved*/) | (7<<5 /*reserved*/) ),
    mask_w   : !((1<<C0_STATUS_BIT_CU3) | (1<<C0_STATUS_BIT_CU2) | (1<<C0_STATUS_BIT_RE) | (1<<C0_STATUS_BIT_SR) | (1<<23 /*reserved*/) | (7<<5 /*reserved*/) ),
    init_val : (1<<C0_STATUS_BIT_BEV) | (1<<C0_STATUS_BIT_ERL),
    const_val: 0,
};

// Debug mode is ignored in this emulator.
#[macro_export]
macro_rules! mode_is_in_error     
{ ( $c0_status:expr ) => (if 0 != ($c0_status & (1<<cp0def::C0_STATUS_BIT_ERL)) { true }else{ false }) }
#[macro_export]
macro_rules! mode_is_in_exception 
{ ( $c0_status:expr ) => (if 0 != ($c0_status & (1<<cp0def::C0_STATUS_BIT_EXL)) { true }else{ false }) }

#[macro_export]
macro_rules! mode_is_exception    
{ ( $c0_status:expr ) => (if 0 != ($c0_status & ((1<<cp0def::C0_STATUS_BIT_EXL) | (1<<cp0def::C0_STATUS_BIT_ERL))) { true }else{ false }) }
#[macro_export]
macro_rules! mode_is_kernel       
{ ( $c0_status:expr ) => ((!mode_is_exception!($c0_status)) && (($c0_status & cp0def::C0_STATUS_KSU_MASK) == (0<<cp0def::C0_STATUS_BIT_KSU))) }
#[macro_export]
macro_rules! mode_is_supervisor   
{ ( $c0_status:expr ) => ((!mode_is_exception!($c0_status)) && (($c0_status & cp0def::C0_STATUS_KSU_MASK) == (1<<cp0def::C0_STATUS_BIT_KSU))) }
#[macro_export]
macro_rules! mode_is_user         
{ ( $c0_status:expr ) => ((!mode_is_exception!($c0_status)) && (($c0_status & cp0def::C0_STATUS_KSU_MASK) == (2<<cp0def::C0_STATUS_BIT_KSU))) }

/*
Exception Code values in ExcCode Field of Cause Register
from "MIPS32 74K Processor Core Family Software User’s Manual," Revision 01.05, March 30, 2011, page 175-176.
*/
pub const C0_CAUSE_BIT_BD     : u32 = 31;    /* Indicates whether the last exception taken occurred in a branch delay slot */
pub const C0_CAUSE_BIT_TI     : u32 = 30;    /* Timer Interrupt (1 when pending) */
pub const C0_CAUSE_BIT_CE     : u32 = 28;    /* Coprocessor unit number referenced when a Coprocessor Unusable exception */
pub const C0_CAUSE_BIT_DC     : u32 = 27;    /* Disable Count register */
pub const C0_CAUSE_BIT_PCI    : u32 = 26;    /* Performance Counter Interrupt */
pub const C0_CAUSE_BIT_IV     : u32 = 23;    /* Indicates whether an interrupt exception uses the general exception vector or a special interrupt vector */
pub const C0_CAUSE_BIT_WP     : u32 = 22;    /* a watch exception was deferred because StatusEXL or StatusERL was a one at the time the watch exception was detected */
pub const C0_CAUSE_BIT_FDCI   : u32 = 21;    /* Fast Debug Channel Interrupt */
pub const C0_CAUSE_BIT_IP     : u32 =  8;    /* pending interrupt or request for software interrupt */
pub const C0_CAUSE_BIT_EXCCODE: u32 =  2;    /* pending interrupt or request for software interrupt */

pub const C0_CAUSE_CE_MASK      : u32 =    3 << C0_CAUSE_BIT_CE;
pub const C0_CAUSE_IP_MASK      : u32 = 0xff << C0_CAUSE_BIT_IP;
pub const C0_CAUSE_EXCCODE_MASK : u32 = 0x1f << C0_CAUSE_BIT_EXCCODE;

const C0_CAUSE_MASK_R: u32 = C0_CAUSE_CE_MASK | C0_CAUSE_IP_MASK | C0_CAUSE_EXCCODE_MASK | 
                            (1<<C0_CAUSE_BIT_BD) | 
                            (1<<C0_CAUSE_BIT_TI) | 
                            (1<<C0_CAUSE_BIT_DC) | 
                            (1<<C0_CAUSE_BIT_IV) | 
                            (1<<C0_CAUSE_BIT_WP); /* PCI, FDCI are ignored */
const C0_CAUSE_MASK_W: u32 = (1<<C0_CAUSE_BIT_DC) | (1<<C0_CAUSE_BIT_IV) | (1<<C0_CAUSE_BIT_WP) | (3<<C0_CAUSE_BIT_IP);

pub const C0_CAUSE_SETTING : C0RegSetting = C0RegSetting {
    mask_r   : C0_CAUSE_MASK_R,
    mask_w   : C0_CAUSE_MASK_W,
    init_val : 0,
    const_val: 0,
};

/*
Exception Code values in ExcCode Field of Cause Register
from "MIPS32 74K Processor Core Family Software User’s Manual," Revision 01.05, March 30, 2011, page 175-176.
*/
pub const EXCEPT_CODE_INTERRUPT             : u32 = 0;  /* Interrupt */  
pub const EXCEPT_CODE_MOD                   : u32 = 1;  /* Store, but page marked as read-only in the TLB */
pub const EXCEPT_CODE_TLB_LOAD              : u32 = 2;  /* Load or fetch, but page marked as invalid in the TLB */
pub const EXCEPT_CODE_TLB_STORE             : u32 = 3;  /* Store, but page marked as invalid in the TLB */
pub const EXCEPT_CODE_ADDR_ERR_LOAD         : u32 = 4;  /* Address error on load/fetch. Address is either wrongly aligned, or a privilege violation. */
pub const EXCEPT_CODE_ADDR_ERR_STORE        : u32 = 5;  /* Address error on store. Address is either wrongly aligned, or a privilege violation. */
pub const EXCEPT_CODE_BUS_ERR_IFETCH        : u32 = 6;  /* Bus error signaled on instruction fetch */
pub const EXCEPT_CODE_BUS_ERR_DATA          : u32 = 7;  /* Bus error signaled on load/store */
pub const EXCEPT_CODE_SYSCALL               : u32 = 8;  /* System call, i.e. syscall instruction executed */
pub const EXCEPT_CODE_BREAKPOINT            : u32 = 9;  /* Breakpoint, i.e. break instruction executed */
pub const EXCEPT_CODE_RESERVED_INSTRUCTION  : u32 =10;  /* Instruction code not recognized (or not legal) */
pub const EXCEPT_CODE_COPROCESSOR_UNAVAIL   : u32 =11;  /* Instruction code was for a co-processor which is not enabled in StatusCU3-0 */
pub const EXCEPT_CODE_INTEGER_OVERFLOW      : u32 =12;  /* Overflow from a trapping variant of integer arithmetic instructions */
pub const EXCEPT_CODE_TRAP                  : u32 =13;  /* Condition met on one of the conditional trap instructions teq etc */
pub const EXCEPT_CODE_FP_EXCEPTION          : u32 =15;  /* Floating point unit exception — more details in the FPU control/status registers */
pub const EXCEPT_CODE_WATCH                 : u32 =23;  /* Instruction or data reference matched a watchpoint */
pub const EXCEPT_CODE_MCHECK                : u32 =24;  /* "Machine check" */
pub const EXCEPT_CODE_THREAD                : u32 =25;  /* Thread-related exception, only for CPUs supporting the MIPS MT ASE */
pub const EXCEPT_CODE_DSP                   : u32 =26;  /* Tried to run an instruction from the MIPS DSP ASE, but it’s either not enabled or not available */
pub const EXCEPT_CODE_CACHE_ERROR           : u32 =30;  /* Parity/ECC error somewhere in the core, on either instruction fetch, load or cache refill */

pub const EXCEPT_CODE_TLB_REFILL_LOAD       : u32 =32;  /* virtual exception code for TLB Refill not used in hardware */
pub const EXCEPT_CODE_TLB_REFILL_STORE      : u32 =33;  /* virtual exception code for TLB Refill not used in hardware */


/*
Definitions of pairs of a register number and a selector number for CP0 registers
and supporting macros.
*/
#[macro_export]
macro_rules! c0_val {
    ( $mem:expr, $c0:expr ) => ($mem.cp0[ ((($c0.0 << mips::CP_SEL_BITS) as u32 + $c0.1) & ((1<<(mips::CP_REG_BITS+mips::CP_SEL_BITS))-1)) as usize ])
}



pub const C0_BADVADDR : (u32,u32) = ( 8, 0);
pub const C0_CACHEERR : (u32,u32) = (27, 0);
pub const C0_CAUSE    : (u32,u32) = (13, 0);
pub const C0_CDMMBASE : (u32,u32) = (15, 2);
pub const C0_CMGCRBASE: (u32,u32) = (15, 3);
pub const C0_COMPARE  : (u32,u32) = (11, 0);
pub const C0_CONFIG   : (u32,u32) = (16, 0);
pub const C0_CONFIG1  : (u32,u32) = (16, 1);
pub const C0_CONFIG2  : (u32,u32) = (16, 2);
pub const C0_CONFIG3  : (u32,u32) = (16, 3);
pub const C0_CONFIG7  : (u32,u32) = (16, 7);
pub const C0_CONTEXT  : (u32,u32) = ( 4, 0);
pub const C0_COUNT    : (u32,u32) = ( 9, 0);
pub const C0_DEBUG    : (u32,u32) = (23, 0);
pub const C0_DEPC     : (u32,u32) = (24, 0);
pub const C0_DESAVE   : (u32,u32) = (31, 0);
pub const C0_DDATALO  : (u32,u32) = (28, 3);
pub const C0_DTAGLO   : (u32,u32) = (28, 2);
pub const C0_EBASE    : (u32,u32) = (15, 1);
pub const C0_ENTRYHI  : (u32,u32) = (10, 0);
pub const C0_ENTRYLO0 : (u32,u32) = ( 2, 0);
pub const C0_ENTRYLO1 : (u32,u32) = ( 3, 0);
pub const C0_EPC      : (u32,u32) = (14, 0);
pub const C0_ERRCTL   : (u32,u32) = (26, 0);
pub const C0_ERROREPC : (u32,u32) = (30, 0);
pub const C0_HWRENA   : (u32,u32) = ( 7, 0);
pub const C0_INDEX    : (u32,u32) = ( 0, 0);
pub const C0_INTCTL   : (u32,u32) = (12, 1);
pub const C0_IDATAHI  : (u32,u32) = (29, 1);
pub const C0_IDATALO  : (u32,u32) = (28, 1);
pub const C0_ITAGLO   : (u32,u32) = (28, 0);
pub const C0_L23DATAHI: (u32,u32) = (29, 5);
pub const C0_L23DATALO: (u32,u32) = (28, 5);
pub const C0_L23TAGLO : (u32,u32) = (28, 4);
pub const C0_LLADDR   : (u32,u32) = (17, 0);
pub const C0_MVPCONF0 : (u32,u32) = ( 0, 2);
pub const C0_MVPCONF1 : (u32,u32) = ( 0, 3);
pub const C0_MVPCONTROL:(u32,u32) = ( 0, 1);
pub const C0_PAGEMASK : (u32,u32) = ( 5, 0);
pub const C0_PERFCNT0 : (u32,u32) = (25, 1);
pub const C0_PERFCNT1 : (u32,u32) = (25, 3);
pub const C0_PERFCNT2 : (u32,u32) = (25, 5);
pub const C0_PERFCNT3 : (u32,u32) = (25, 7);
pub const C0_PERFCTL0 : (u32,u32) = (25, 0);
pub const C0_PERFCTL1 : (u32,u32) = (25, 2);
pub const C0_PERFCTL2 : (u32,u32) = (25, 4);
pub const C0_PERFCTL3 : (u32,u32) = (25, 6);
pub const C0_PRID     : (u32,u32) = (15, 0);
pub const C0_RANDOM   : (u32,u32) = ( 1, 0);
pub const C0_SRSCONF0 : (u32,u32) = ( 6, 1);
pub const C0_SRSCONF1 : (u32,u32) = ( 6, 2);
pub const C0_SRSCONF2 : (u32,u32) = ( 6, 3);
pub const C0_SRSCONF3 : (u32,u32) = ( 6, 4);
pub const C0_SRSCONF4 : (u32,u32) = ( 6, 5);
pub const C0_SRSCTL   : (u32,u32) = (12, 2);
pub const C0_SRSMAP   : (u32,u32) = (12, 3);
pub const C0_STATUS   : (u32,u32) = (12, 0);
pub const C0_TCBIND   : (u32,u32) = ( 2, 2);
pub const C0_TCCONTEXT: (u32,u32) = ( 2, 5);
pub const C0_TCHALT   : (u32,u32) = ( 2, 4);
pub const C0_TCRESTART: (u32,u32) = ( 2, 3);
pub const C0_TCSCHEDULE:(u32,u32) = ( 2, 6);
pub const C0_VPESCHEDULE:(u32,u32)= ( 1, 5);
pub const C0_TCSCHEFBACK:(u32,u32)= ( 2, 7);
pub const C0_TCSTATUS : (u32,u32) = ( 2, 1);
pub const C0_TRACECONTROL :(u32,u32)=(23,1);
pub const C0_TRACECONTROL2:(u32,u32)=(23,2);
pub const C0_TRACEDBPC: (u32,u32) = ( 3, 5);
pub const C0_TRACEIBPC: (u32,u32) = ( 3, 4);
pub const C0_USERLOCAL: (u32,u32) = ( 4, 2);
pub const C0_USERTRACEDATA:(u32,u32)=(23,3);
pub const C0_VPECONF0 : (u32,u32) = ( 1, 2);
pub const C0_VPECONF1 : (u32,u32) = ( 1, 3);
pub const C0_VPECONTROL:(u32,u32) = ( 1, 1);
pub const C0_VPEOPT   : (u32,u32) = ( 1, 7);
pub const C0_WATCHHI0 : (u32,u32) = (19, 0);
pub const C0_WATCHHI1 : (u32,u32) = (19, 1);
pub const C0_WATCHHI2 : (u32,u32) = (19, 2);
pub const C0_WATCHHI3 : (u32,u32) = (19, 3);
pub const C0_WATCHLO0 : (u32,u32) = (18, 0);
pub const C0_WATCHLO1 : (u32,u32) = (18, 1);
pub const C0_WATCHLO2 : (u32,u32) = (18, 2);
pub const C0_WATCHLO3 : (u32,u32) = (18, 3);
pub const C0_WIRED    : (u32,u32) = ( 6, 0);
pub const C0_YQMASK   : (u32,u32) = ( 1, 4);





