use crate::dev_soc;
use crate::{exec_mips16, exec_mips32};
use crate::procstate::MachineState;
use crate::mips;
use crate::config;
use crate::cp0def;
use crate::cp0;
use crate::mem;
use crate::exception;
use crate::dev_uart;
use crate::procstate;
use crate::c0_val;
use crate::mode_is_exception;
use log::info;


#[cfg(not(target_family = "wasm"))]
use {std::sync::Arc, std::sync::atomic, std::io::stdout, std::time::Instant, termion::raw::IntoRawMode};


use {crate::wasm_utils, js_sys::Date};


pub async fn run_wasm(ms: &mut MachineState) {
    let mut inst    : u32;
    let mut prev_exec_insts: u64 = 0;
    let mut prev_delay: u64      = 0;

    ms.emu.nexec_insts = 0;
    ms.emu.stopcount   = 0;

    ms.emu.debug = false;

    loop {
        ms.reg.r[0] = 0;
        inst = mem::fetch_instruction(ms);

        if 0 == (ms.reg.pc & 1) {
            if ! exec_mips32::exec(ms, inst) { break; }
        }else{
            if ! exec_mips16::exec(ms, inst) { break; }
        }

        if ms.emu.nexec_insts > prev_exec_insts + 10000  {
            let currenttime :u64 = (Date::now() as u64)*1000;

            if currenttime > prev_delay + 20000 || ms.sleep_req {
                wasm_utils::sleep(1).await;
                prev_delay = currenttime;
                ms.sleep_req = false;
            }

            // Calculating the instruction execution rate
            ms.emu.execrate = (ms.emu.nexec_insts - prev_exec_insts)*((1000*1000 / config::SYSTEM_TIMER_INTERVAL_IN_USEC) as u64);
            prev_exec_insts = ms.emu.nexec_insts;

            // Updating the CoProcessor0 Counter
            ms.reg.c0_count_currenttime    = currenttime;
            ms.reg.c0_count_ninst_in_ctime = ms.emu.nexec_insts;
            dev_uart::read_reg(&mut ms.uart, &mut ms.stdin_ch, dev_uart::IOADDR_UART0_BASE + dev_uart::UART_REG_LINESTAT); // to update internal state

            if ms.misc.reset_request {
                break;
            }
        }
        if ms.reg.c0_compare_long <= cp0::load_counter_long(ms) {
            c0_val!(ms.reg,cp0def::C0_CAUSE) |= (1<<cp0def::C0_INTCTL_TIMER_INT_IPNUM)<<cp0def::C0_CAUSE_BIT_IP;
            c0_val!(ms.reg,cp0def::C0_CAUSE) |=  1<<cp0def::C0_CAUSE_BIT_TI;
        }

        if 0!=ms.uart.int_enable && 0!=((1<<3) & ms.misc.int_mask & dev_soc::read_misc_int_status_reg(ms)) {
            c0_val!(ms.reg,cp0def::C0_CAUSE) |=   (1<<6)<<cp0def::C0_CAUSE_BIT_IP;
        }else{
            c0_val!(ms.reg,cp0def::C0_CAUSE) &= !((1<<6)<<cp0def::C0_CAUSE_BIT_IP);
        }

        let status = c0_val!(ms.reg, cp0def::C0_STATUS);
        if 0!=(status & (1<<cp0def::C0_STATUS_BIT_IE)) && !mode_is_exception!(status) {
            if 0 != (c0_val!(ms.reg, cp0def::C0_CAUSE) & status & cp0def::C0_CAUSE_IP_MASK) {
                exception::prepare_interrupt(ms, (c0_val!(ms.reg, cp0def::C0_CAUSE) & cp0def::C0_CAUSE_IP_MASK) >> cp0def::C0_CAUSE_BIT_IP );
            }
        }

        ms.emu.nexec_insts+=1;
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn run_term(ms: &mut MachineState) {
    let mut pointer : u32 = ms.reg.pc;
    let mut inst    : u32;
    let mut m32mode : bool;
    let mut prev_exec_insts: u64 = 0;

    let start: Instant = Instant::now();
    let ctrlc_num = Arc::clone(&ms.ctrlc_count);
    let time_trig = Arc::clone(&ms.time_trigger); 
    let mut prev_ctrlc_num: usize     = 0;
    let mut prev_ctrlc_trig_time: u64 = 0;

    let _stdout = stdout().into_raw_mode().unwrap();

    ms.emu.nexec_insts = 0;
    ms.emu.stopcount   = 0;

    ms.emu.debug = false;

    /*
    ms.emu.breakpoint = 0x800e55ac;
    ms.emu.runafterbreak = 0x1000;
    */

    while ms.emu.stopcount == 0 || (ms.emu.stopcount > 0 && ms.emu.stopcount >= ms.emu.nexec_insts) {
        ms.reg.r[0] = 0;

        inst = mem::fetch_instruction(ms);
        pointer = ms.reg.pc;
        m32mode = if 0==(pointer&1) { true }else{ false };

        if ms.emu.debug {
            info!("================================== \r");
            info!("pointer: {:>08x}  insts = {:>08x} \r", pointer, inst );
            procstate::log_print_reg32(&ms.reg);
        }
        if ((pointer&ms.emu.breakmask) == (ms.emu.breakpoint&ms.emu.breakmask) && ms.emu.stopcount==0) || (ms.emu.breakcounter != 0 && ms.emu.breakcounter == ms.emu.nexec_insts) {
            info!("Breakpoint\r");
            info!("================================== \r");
            info!("pointer: {:>08x}  insts = {:>08x} \r", pointer, inst);
            procstate::log_print_reg32(&ms.reg);
    
            ms.emu.debug = true;
            ms.emu.stopcount = ms.emu.nexec_insts + ms.emu.runafterbreak;
        }

/*
saveInstPointer(pointer);
*/
        if m32mode {
            if ! exec_mips32::exec(ms, inst) {
                break;
            }
        }else{
            if ! exec_mips16::exec(ms, inst) {
                break;
            }
        }

        if time_trig.load(atomic::Ordering::Relaxed)  {
            time_trig.swap(false, atomic::Ordering::Relaxed);
            let currenttime :u64 = start.elapsed().as_micros() as u64;

            // Calculating the instruction execution rate
            ms.emu.execrate = (ms.emu.nexec_insts - prev_exec_insts)*((1000*1000 / config::SYSTEM_TIMER_INTERVAL_IN_USEC) as u64);
            prev_exec_insts = ms.emu.nexec_insts;

            // Updating the CoProcessor0 Counter
            ms.reg.c0_count_currenttime    = currenttime;
            ms.reg.c0_count_ninst_in_ctime = ms.emu.nexec_insts;
            dev_uart::read_reg(&mut ms.uart, &mut ms.stdin_ch, dev_uart::IOADDR_UART0_BASE + dev_uart::UART_REG_LINESTAT); // to update internal state


            // Checking the SIGINT status, treatment of Ctrl+C.
            if ctrlc_num.load(atomic::Ordering::Relaxed) != prev_ctrlc_num {
                // time between two Ctrl+C keyins is shorter than 1000ms, then enter the monitor
                if currenttime - prev_ctrlc_trig_time < 1000*1000 {
                    /*
                    if( emuMonitor(pM) < 0 ){
                        // halt the system
                        pM->mem.ioMISC.reset_request = 1;
                    }
                    */
                    ms.misc.reset_request = true;
                    prev_ctrlc_trig_time = 0;
                }else{
                    prev_ctrlc_trig_time = currenttime;
                }
                prev_ctrlc_num = ctrlc_num.load(atomic::Ordering::Relaxed);
            }
            if prev_ctrlc_trig_time != 0 &&  currenttime - prev_ctrlc_trig_time >= 1000*1000 {
                dev_uart::request_send_break(&mut ms.uart);
                prev_ctrlc_trig_time = 0;
            }



            if ms.misc.reset_request /*|| ctrlc_num.load(atomic::Ordering::Relaxed) > 0*/ {
                print!("\r\n\r\n");
                info!("System reset ...\r");
                break;
            }
        }
        if ms.reg.c0_compare_long <= cp0::load_counter_long(ms) {
            c0_val!(ms.reg,cp0def::C0_CAUSE) |= (1<<cp0def::C0_INTCTL_TIMER_INT_IPNUM)<<cp0def::C0_CAUSE_BIT_IP;
            c0_val!(ms.reg,cp0def::C0_CAUSE) |=  1<<cp0def::C0_CAUSE_BIT_TI;
        }

        if 0!=ms.uart.int_enable && 0!=((1<<3) & ms.misc.int_mask & dev_soc::read_misc_int_status_reg(ms)) {
            c0_val!(ms.reg,cp0def::C0_CAUSE) |=   (1<<6)<<cp0def::C0_CAUSE_BIT_IP;
        }else{
            c0_val!(ms.reg,cp0def::C0_CAUSE) &= !((1<<6)<<cp0def::C0_CAUSE_BIT_IP);
        }

        let status = c0_val!(ms.reg, cp0def::C0_STATUS);
        if 0!=(status & (1<<cp0def::C0_STATUS_BIT_IE)) && !mode_is_exception!(status) {
            if 0 != (c0_val!(ms.reg, cp0def::C0_CAUSE) & status & cp0def::C0_CAUSE_IP_MASK) {
                exception::prepare_interrupt(ms, (c0_val!(ms.reg, cp0def::C0_CAUSE) & cp0def::C0_CAUSE_IP_MASK) >> cp0def::C0_CAUSE_BIT_IP );
            }
        }

        ms.emu.nexec_insts+=1;
    }

    info!("pointer 0x{:>x}\r", pointer);
}
