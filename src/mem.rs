use crate::{config, procstate::MachineState};
use crate::{dev_spiflash, mips};
use crate::exec_mips16;
use crate::dev_uart;
use crate::dev_soc;
use crate::dev_spi;
use crate::cp0def;
use crate::tlb;
use crate::exception;
use crate::kseg01_to_paddr;
use crate::mode_is_in_error;
use crate::mode_is_exception;
use crate::mode_is_user;
use crate::mode_is_supervisor;
use crate::c0_val;

pub struct MemRegion {
    pub mem0 : Box<[u8]>
}

impl MemRegion {
    pub fn new() -> Self {
        Self { 
            mem0: vec![0xff as u8; config::DRAM_SIZE].into_boxed_slice(),
        }
    }
}

fn read_phys_mem_word(ms : &mut MachineState, addr:u32) -> u32 {

    if addr >= config::RAM_AREA_ADDR && addr+3 < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {
        let addr0 : usize = ((addr + 0) & config::DRAM_ADDR_MASK) as usize;
        let addr1 : usize = ((addr + 1) & config::DRAM_ADDR_MASK) as usize;
        let addr2 : usize = ((addr + 2) & config::DRAM_ADDR_MASK) as usize;
        let addr3 : usize = ((addr + 3) & config::DRAM_ADDR_MASK) as usize;

        return ((ms.mem.mem0[addr0] as u32) <<24) |
               ((ms.mem.mem0[addr1] as u32) <<16) |
               ((ms.mem.mem0[addr2] as u32) << 8) |
               ((ms.mem.mem0[addr3] as u32)     ) ;
    }

    if addr >= config::ROM_AREA_ADDR && addr+3 < config::ROM_AREA_ADDR+config::ROM_AREA_SIZE {
        let spi_addr:u32 = addr - config::ROM_AREA_ADDR;
        let mut data:u32 = 0;
        ms.spi.workers[0].select();
        ms.spi.workers[0].write( dev_spiflash::FLASH_CMD_READ );
        ms.spi.workers[0].write( ((spi_addr>>16) & 0xff) as u8 );
        ms.spi.workers[0].write( ((spi_addr>> 8) & 0xff) as u8 );
        ms.spi.workers[0].write( ((spi_addr>> 0) & 0xff) as u8 );
        data |= ms.spi.workers[0].write( 0xff ) as u32; data = data<<8;
        data |= ms.spi.workers[0].write( 0xff ) as u32; data = data<<8;
        data |= ms.spi.workers[0].write( 0xff ) as u32; data = data<<8;
        data |= ms.spi.workers[0].write( 0xff ) as u32; 
        ms.spi.workers[0].deselect();

        return data;
    }

    return 0;
}

fn write_phys_mem_word(ms : &mut MachineState, addr : u32, data : u32){

    if addr >= config::RAM_AREA_ADDR && addr+3 < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {
        let addr0 : usize = ((addr + 0) & config::DRAM_ADDR_MASK) as usize;
        let addr1 : usize = ((addr + 1) & config::DRAM_ADDR_MASK) as usize;
        let addr2 : usize = ((addr + 2) & config::DRAM_ADDR_MASK) as usize;
        let addr3 : usize = ((addr + 3) & config::DRAM_ADDR_MASK) as usize;

        ms.mem.mem0[addr0] = ((data>>24)&0xff) as u8;
        ms.mem.mem0[addr1] = ((data>>16)&0xff) as u8;
        ms.mem.mem0[addr2] = ((data>> 8)&0xff) as u8;
        ms.mem.mem0[addr3] = ((data    )&0xff) as u8;
        return;
    }
}

fn read_phys_mem_halfword(ms : &mut MachineState, addr : u32) -> u32{

    if addr >= config::RAM_AREA_ADDR && addr+1 < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {
        let addr0 : usize = ((addr + 0) & config::DRAM_ADDR_MASK) as usize;
        let addr1 : usize = ((addr + 1) & config::DRAM_ADDR_MASK) as usize;

        return ((ms.mem.mem0[addr0] as u32) <<8) |
               ((ms.mem.mem0[addr1] as u32)    ) ;
    }

    if addr >= config::ROM_AREA_ADDR && addr+1 < config::ROM_AREA_ADDR+config::ROM_AREA_SIZE {
        let spi_addr:u32 = addr - config::ROM_AREA_ADDR;
        let mut data:u32 = 0;

        ms.spi.workers[0].select();
        ms.spi.workers[0].write( dev_spiflash::FLASH_CMD_READ );
        ms.spi.workers[0].write( ((spi_addr>>16) & 0xff) as u8 );
        ms.spi.workers[0].write( ((spi_addr>> 8) & 0xff) as u8 );
        ms.spi.workers[0].write( ((spi_addr>> 0) & 0xff) as u8 );
        data |= ms.spi.workers[0].write( 0xff ) as u32; data = data<<8;
        data |= ms.spi.workers[0].write( 0xff ) as u32; 
        ms.spi.workers[0].deselect();

        return data;
    }
    return 0;
}

fn write_phys_mem_halfword(ms : &mut MachineState, addr : u32, data : u32){

    if addr >= config::RAM_AREA_ADDR && addr+1 < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {
        let addr0 : usize = ((addr + 0) & config::DRAM_ADDR_MASK) as usize;
        let addr1 : usize = ((addr + 1) & config::DRAM_ADDR_MASK) as usize;

        ms.mem.mem0[addr0] = ((data>>8)&0xff) as u8;
        ms.mem.mem0[addr1] = ((data   )&0xff) as u8;
        return;
    }
}

fn read_phys_mem_byte(ms : &mut MachineState, addr : u32) -> u8{

    if addr >= config::RAM_AREA_ADDR && addr < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {
        let addr0 : usize = ((addr + 0) & config::DRAM_ADDR_MASK) as usize;
        return ms.mem.mem0[addr0];
    }

    if addr >= config::ROM_AREA_ADDR && addr < config::ROM_AREA_ADDR+config::ROM_AREA_SIZE {
        let spi_addr:u32 = addr - config::ROM_AREA_ADDR;
        let data:u8;

        ms.spi.workers[0].select();
        ms.spi.workers[0].write( dev_spiflash::FLASH_CMD_READ );
        ms.spi.workers[0].write( ((spi_addr>>16) & 0xff) as u8 );
        ms.spi.workers[0].write( ((spi_addr>> 8) & 0xff) as u8 );
        ms.spi.workers[0].write( ((spi_addr>> 0) & 0xff) as u8 );
        data = ms.spi.workers[0].write( 0xff ); 
        ms.spi.workers[0].deselect();

        return data;
    }

    return 0;
}

fn write_phys_mem_byte(ms : &mut MachineState, addr : u32, data : u8){

    if addr >= config::RAM_AREA_ADDR && addr < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {
        let addr0 : usize = ((addr + 0) & config::DRAM_ADDR_MASK) as usize;
        ms.mem.mem0[addr0] = data;
        return;
    }
}

fn get_phy_addr(ms : &mut MachineState, addr: u32, is_write: bool) -> Result<u32, u32> {
    let c0_status : u32 = c0_val!(ms.reg, cp0def::C0_STATUS);
    let asid      : u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI) & cp0def::C0_ENTRYHI_ASID_MASK;

    if addr < mips::KSEG0 {
        if mode_is_in_error!(c0_status) && (addr < (1<<29)) {
            return Ok(addr);
        }else{
            return tlb::lookup(ms, asid, is_write, addr);
        }
    }

    if mode_is_user!( c0_status ) {
        return Err( if is_write { cp0def::EXCEPT_CODE_ADDR_ERR_STORE }else{ cp0def::EXCEPT_CODE_ADDR_ERR_LOAD } );
    }

    if addr >= mips::KSEG0 && addr < mips::KSEG1 {
        if mode_is_supervisor!( c0_status ) {
            return if is_write { Err(cp0def::EXCEPT_CODE_ADDR_ERR_STORE) }else{ Err(cp0def::EXCEPT_CODE_ADDR_ERR_LOAD) };
        }

        let paddr = kseg01_to_paddr!(addr);
        if (0 == ((ms.spi.control) & (1<<dev_spi::SPI_CTRL_BIT_REMAP_DISABLE))) && paddr >= kseg01_to_paddr!(mips::EXCEPT_VECT_RESET) {
            /*
              Memory image
              [1fc0_0000 - 1fff_ffff] -> [0x1f00_0000 - 0x1f3f_ffff]
            */
            return Ok(paddr & 0xff3fffff);   // memory image
        }else{
            return Ok(paddr);
        }
    }

    if addr >= mips::KSEG1 && addr < mips::KSEG2 {
        if mode_is_supervisor!( c0_status ) {
            return if is_write { Err(cp0def::EXCEPT_CODE_ADDR_ERR_STORE) }else{ Err(cp0def::EXCEPT_CODE_ADDR_ERR_LOAD) };
        }

        let paddr = kseg01_to_paddr!(addr);
        if (0 == ((ms.spi.control) & (1<<dev_spi::SPI_CTRL_BIT_REMAP_DISABLE))) && paddr >= kseg01_to_paddr!(mips::EXCEPT_VECT_RESET) {
            /*
              Memory image
              [1fc0_0000 - 1fff_ffff] -> [0x1f00_0000 - 0x1f3f_ffff]
            */
            return Ok(paddr & 0xff3fffff);   // memory image
        }else{
            return Ok(paddr);
        }
    }

    if addr >= mips::KSEG2 && addr < mips::KSEG3 {
        return tlb::lookup(ms, asid, is_write, addr);
    }

    if addr >= mips::KSEG3 {
        if mode_is_supervisor!( c0_status ) {
            return if is_write { Err(cp0def::EXCEPT_CODE_ADDR_ERR_STORE) }else{ Err(cp0def::EXCEPT_CODE_ADDR_ERR_LOAD) };
        }
        return tlb::lookup(ms, asid, is_write, addr);
    }

    // never reaches
    return Ok(addr);
}


pub fn fetch_instruction(ms : &mut MachineState) -> u32{

    if (ms.reg.pc & 3) == 2 {
        // If the lower 2 bits of PC is 10, PC value is misaligned.
        //   ...00 is valid MIPS32 address.
        //   ...01 and ...11 are valid MIPS16 address. 
        //   Note that the LSB, i.e., 1, is the mode bit representing MIPS16 mode.
        exception::prepare_exception(ms, cp0def::EXCEPT_CODE_ADDR_ERR_LOAD, ms.reg.pc);
        return fetch_instruction(ms);
    }

    let asid : u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI) & cp0def::C0_ENTRYHI_ASID_MASK;
    let mode : u32 = c0_val!(ms.reg, cp0def::C0_STATUS)  & (cp0def::C0_STATUS_KSU_MASK | (1<<cp0def::C0_STATUS_BIT_ERL) | (1<<cp0def::C0_STATUS_BIT_EXL));

    /*
     * In a 4K-byte page, physical address is linear.
     * This function generates physical address using the cached pair 
     * of a previous virtual address and its corresonding physical page.
     * The mapping from virtual to physical may change 
     * when TLB-write occurs or the setting of memory-remap changes.
     * Thus, this cache is cleared in TLBWrite function and writing to SPI registers.
     */
    let paddr:u32;
    if ms.reg.pc_cache.check( ms.reg.pc, asid, mode) {
        paddr = ms.reg.pc_cache.get_addr(ms.reg.pc);
    }else{
        match get_phy_addr(ms, ms.reg.pc, false){
            Ok(phy_addr) => {
                ms.reg.pc_cache.set(ms.reg.pc, asid, mode, phy_addr);
                paddr = phy_addr;
            }
            Err(ecode) => { 
                exception::prepare_exception(ms, ecode, ms.reg.pc); 
                return fetch_instruction(ms); 
            }
        };
    }


    if 0 != (ms.reg.pc & 1) {
        let inst:u32 = read_phys_mem_halfword(ms, paddr&(!1));
        let op : u32 = (inst>>11) & 0x1f;

        if op == exec_mips16::MIPS16E_OP_EXTEND || op == exec_mips16::MIPS16E_OP_JAL {
            let nextaddr : u32 = ms.reg.pc + 1;
            match get_phy_addr(ms, nextaddr, false)
            {
                Ok(phy_addr)  =>{
                    return (inst << 16) | read_phys_mem_halfword(ms, phy_addr);
                }
                Err(ecode) => { 
                    exception::prepare_exception(ms, ecode, nextaddr); 
                    return fetch_instruction(ms); 
                }
            };
        }
        return inst;
    }

    return read_phys_mem_word(ms, paddr&(!3));

}




fn accsize_align(width : u32, addr : u32, val : u32) -> u32{
    return match width 
    {
        2 =>
        {
            match addr & 1{
                0 => (val>>16)&0xffff,
                _ =>  val&0xffff,
            }
        }
        1 =>
        {
            match addr & 3 {
                0 => (val>>24)&0xff,
                1 => (val>>16)&0xff,
                2 => (val>> 8)&0xff,
                _ => (val>> 0)&0xff,
            }
        }
        _ => val,
    }
}

fn wrdata_align(width : u32, addr : u32, val : u32) -> u32{
    return match width 
    {
        2=>
        {
            match addr & 1 {
                0 => (val<<16)&0xffff0000,
                _ => val&0xffff,
            }
        }
        1=>
        {
            match addr & 3 {
                0 => (val<<24)&0xff000000,
                1 => (val<<16)&0x00ff0000,
                2 => (val<< 8)&0x0000ff00,
                _ => (val<< 0)&0x000000ff,
            }
        }
        _ => val,
    }
}

fn load_memory(ms : &mut MachineState, vaddr : u32, acc_width : u32) -> Result<u32,u32> { 
    let asid : u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI) & cp0def::C0_ENTRYHI_ASID_MASK;
    let mode : u32 = c0_val!(ms.reg, cp0def::C0_STATUS)  & (cp0def::C0_STATUS_KSU_MASK | (1<<cp0def::C0_STATUS_BIT_ERL) | (1<<cp0def::C0_STATUS_BIT_EXL));

    /*
     * In a 4K-byte page, physical address is linear.
     * This function generates physical address using the cached pair 
     * of a previous virtual address and its corresonding physical page.
     * The mapping from virtual to physical may change 
     * when TLB-write occurs or the setting of memory-remap changes.
     * Thus, this cache is cleared in TLBWrite function and writing to SPI registers.
     */
    let paddr:u32;

    if vaddr & 0xf0000000 == 0 {
        if ms.reg.dr_cache[0].check( vaddr, asid, mode) {
            paddr = ms.reg.dr_cache[0].get_addr(vaddr);
        }else{
            paddr = get_phy_addr(ms, vaddr, false)?;
            ms.reg.dr_cache[0].set(vaddr, asid, mode, paddr);
        }
    }else{
        if ms.reg.dr_cache[1].check( vaddr, asid, mode) {
            paddr = ms.reg.dr_cache[1].get_addr(vaddr);
        }else{
            paddr = get_phy_addr(ms, vaddr, false)?;
            ms.reg.dr_cache[1].set(vaddr, asid, mode, paddr);
        }
    }

    let align_addr :u32 = paddr & !(3 as u32);

    if paddr >= config::RAM_AREA_ADDR && paddr < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {

        let val:u32 = match acc_width{
            1 => read_phys_mem_byte(ms, paddr) as u32,
            2 => read_phys_mem_halfword(ms, paddr),
            _ => read_phys_mem_word(ms, paddr),
        };
        return Ok(val);

    }else if paddr >= config::ROM_AREA_ADDR && paddr < config::ROM_AREA_ADDR+config::ROM_AREA_SIZE {
        // SPI peripheral or SPI flash

        if 0 != (dev_spi::read_reg(&ms.spi, dev_spi::SPI_FUNC_SEL_REG) & 1) {
            // GPIO mode is enabled. FLASH memory is not mapped to memory area.
            // Peripheral registers are visible.
            if paddr >= dev_spi::SPI0_BASE_ADDRESS && paddr < dev_spi::SPI0_BASE_ADDRESS+dev_spi::SPI_ADDR_SIZE {
                return Ok( accsize_align(acc_width, paddr, dev_spi::read_reg(&ms.spi, align_addr)) );
            }else{
                return Ok(0);
            }
        }else{
            // GPIO mode is disabled. SPI flash memory data is mapped to this region.
            // Peripheral registers are not visible.
            let val:u32 = match acc_width{
                1 => read_phys_mem_byte(ms, paddr) as u32,
                2 => read_phys_mem_halfword(ms, paddr),
                _ => read_phys_mem_word(ms, paddr),
            };
            return Ok(val);
        }

    }else if paddr >= dev_uart::IOADDR_UART0_BASE && paddr < dev_uart::IOADDR_UART0_BASE+dev_uart::IOADDR_UART_SIZE {
        // UART

        return Ok( accsize_align(acc_width, paddr, dev_uart::read_reg(&mut ms.uart, &mut ms.stdin_ch, align_addr) as u32));

    }else if paddr >= dev_soc::GPIO_BASE_REG && paddr < dev_soc::GPIO_BASE_REG+0x100 {
        // GPIO

        return match align_addr - dev_soc::GPIO_BASE_REG {
            0x00 /*GPIO_OE */ => Ok(accsize_align(acc_width, paddr, ms.gpio.oe)),
            0x08 /*GPIO_OUT*/ => Ok(accsize_align(acc_width, paddr, ms.gpio.out)),
            _                 => Ok(0)
        };

    }else if paddr >= dev_soc::RTC_BASE_REG && paddr < dev_soc::RTC_BASE_REG+0x5c {
        // RTC

        return match align_addr - dev_soc::RTC_BASE_REG {
            0x44 => Ok( accsize_align(acc_width, paddr, 2) ),
            _    => Ok(0),
        };

    }else if paddr >= dev_soc::RST_BASE_REG && paddr < dev_soc::RST_BASE_REG+0x100 {
        // RESET

        return match align_addr {
            dev_soc::RST_MISC_INTERRUPT_STATUS_REG => Ok( accsize_align(acc_width, paddr, dev_soc::read_misc_int_status_reg(ms)) ),
            dev_soc::RST_BOOTSTRAP_REG             => Ok( accsize_align(acc_width, paddr, (7<<8) | (1<<2) | (1<<4)) ), // Reference clock : 40MHz
            dev_soc::RST_REVISION_ID_REG           => Ok( accsize_align(acc_width, paddr, dev_soc::RST_REVISION_ID_MAJOR_AR9342_VAL | 3) ), // SOC index (AR9342)
            dev_soc::RST_MISC_INTERRUPT_MASK_REG   => Ok( accsize_align(acc_width, paddr, ms.misc.int_mask) ),
            _                                      => Ok(0)
        };

    }else if paddr >= dev_soc::PLL_BASE_REG && paddr < dev_soc::PLL_BASE_REG+0x100 {
        // PLL

        return match align_addr {
            dev_soc::PLL_CPU_DDR_CLK_CTRL_REG => Ok( accsize_align(acc_width, paddr, 1<<20) ), // CPU clock from CPU PLL
            _                                 => Ok(0)
        };

    }else if paddr >= dev_soc::PLL_SRIF_CPU_DPLL_BASE_REG && paddr < dev_soc::PLL_SRIF_CPU_DPLL_BASE_REG+0x100 {
        // SRIF

        return match align_addr {
            dev_soc::PLL_SRIF_CPU_DPLL1_REG => Ok( accsize_align(acc_width, paddr, (1<<27 /*refdiv*/) + (10<<18 /*nint*/) + (0 /*nfrac*/)) ),
            dev_soc::PLL_SRIF_CPU_DPLL2_REG => Ok( accsize_align(acc_width, paddr, (1<<30) + (0<<13 /*outdiv*/)) ),
            _                                 => Ok(0)
        };
    }

    //printf("[memory read: 0x%x]\n", addr);

    Ok(0)
}

fn store_memory(ms : &mut MachineState, vaddr : u32, acc_width: u32, data : u32) -> Result<(),u32> {
    let asid : u32 = c0_val!(ms.reg, cp0def::C0_ENTRYHI) & cp0def::C0_ENTRYHI_ASID_MASK;
    let mode : u32 = c0_val!(ms.reg, cp0def::C0_STATUS)  & (cp0def::C0_STATUS_KSU_MASK | (1<<cp0def::C0_STATUS_BIT_ERL) | (1<<cp0def::C0_STATUS_BIT_EXL));

    /*
     * In a 4K-byte page, physical address is linear.
     * This function generates physical address using the cached pair 
     * of a previous virtual address and its corresonding physical page.
     * The mapping from virtual to physical may change 
     * when TLB-write occurs or the setting of memory-remap changes.
     * Thus, this cache is cleared in TLBWrite function and writing to SPI registers.
     */

    let paddr : u32;

    if vaddr & 0xf0000000 == 0 {
        if ms.reg.dw_cache[0].check( vaddr, asid, mode) {
            paddr = ms.reg.dw_cache[0].get_addr(vaddr);
        }else{
            paddr = get_phy_addr(ms, vaddr, true)?;
            ms.reg.dw_cache[0].set(vaddr, asid, mode, paddr);
        }
    }else{
        if ms.reg.dw_cache[1].check( vaddr, asid, mode) {
            paddr = ms.reg.dw_cache[1].get_addr(vaddr);
        }else{
            paddr = get_phy_addr(ms, vaddr, true)?;
            ms.reg.dw_cache[1].set(vaddr, asid, mode, paddr);
        }
    }


    let align_addr :u32 = paddr & !(3 as u32);

    if paddr >= config::RAM_AREA_ADDR && paddr < config::RAM_AREA_ADDR+config::RAM_AREA_SIZE {

        match acc_width{
            1=> { write_phys_mem_byte(ms, paddr, data as u8); }
            2=> { write_phys_mem_halfword(ms, paddr, data);}
            _=> { write_phys_mem_word(ms, paddr, data);}
        }
        return Ok(());


    }else if paddr >= dev_spi::SPI0_BASE_ADDRESS && paddr <= dev_spi::SPI0_BASE_ADDRESS+dev_spi::SPI_ADDR_SIZE {
        // SPI

        /* 
         * Writing SPI register, SPI_CONTROL_ADDR, may change memory mapping.
         * Therefore, address caches should be cleared.
         */
        if paddr >= dev_spi::SPI0_BASE_ADDRESS + dev_spi::SPI_CTRL_REG && paddr < dev_spi::SPI0_BASE_ADDRESS + dev_spi::SPI_CTRL_REG + 4 &&
            ( (ms.spi.control ^ data) & (1<<dev_spi::SPI_CTRL_BIT_REMAP_DISABLE) ) != 0 {
            ms.reg.pc_cache.clear();
            ms.reg.dr_cache[0].clear();
            ms.reg.dr_cache[1].clear();
            ms.reg.dw_cache[0].clear();
            ms.reg.dw_cache[1].clear();
        }

        dev_spi::write_reg(&mut ms.spi, align_addr, wrdata_align(acc_width, paddr, data) );
        return Ok(());

    }else if paddr >= dev_uart::IOADDR_UART0_BASE && paddr < dev_uart::IOADDR_UART0_BASE+dev_uart::IOADDR_UART_SIZE {
        // UART

        dev_uart::write_reg(&mut ms.uart, &mut ms.stdin_ch, align_addr, wrdata_align(acc_width, paddr, data) as u8);
        return Ok(());
    }else if paddr >= dev_soc::GPIO_BASE_REG && paddr < dev_soc::GPIO_BASE_REG+0x100 {
        // GPIO
        match align_addr - dev_soc::GPIO_BASE_REG {
            0x00 /*GPIO_OE*/  => { ms.gpio.oe  = wrdata_align(acc_width, paddr, data);  }
            0x08 /*GPIO_OUT*/ => { ms.gpio.out = wrdata_align(acc_width, paddr, data);  }
            0x0C /*GPIO_SET*/ => { ms.gpio.out|= wrdata_align(acc_width, paddr, data);  }
            0x10 /*GPIO_CLR*/ => { ms.gpio.out&=!wrdata_align(acc_width, paddr, data);  }
            _                 => { }
        }
    }else if paddr >= dev_soc::RST_BASE_REG && paddr < dev_soc::RST_BASE_REG+0x100 {
        // RESET
        match align_addr {
            dev_soc::RST_MISC_INTERRUPT_MASK_REG => { ms.misc.int_mask = data; }
            dev_soc::RST_RESET_REG               => { ms.misc.reset_request = if 0!=(data & (1<<24)) { true }else{ false }; /* FULL CHIP RESET */ }
            _  => { }
        }
    }

    //printf("[memory write: 0x%x @ 0x%x]\n", data, addr);
    Ok(())
}

pub fn load_byte(ms : &mut MachineState, addr : u32) -> Result<u32,u32> {
    return load_memory(ms, addr, 1);
}
pub fn load_halfword(ms : &mut MachineState, addr : u32) -> Result<u32,u32> { 
    if (addr&1) != 0 {
        return Err( cp0def::EXCEPT_CODE_ADDR_ERR_LOAD );
    }
    return load_memory(ms, addr, 2);
}
pub fn load_word(ms : &mut MachineState, addr : u32) -> Result<u32,u32> { 
    if (addr&3) != 0 {
        return Err( cp0def::EXCEPT_CODE_ADDR_ERR_LOAD );
    }
    return load_memory(ms, addr, 4);
}

pub fn store_byte(ms : &mut MachineState, addr : u32, data :u8) -> Result<(),u32> {
    return store_memory(ms, addr, 1, data as u32);
}

pub fn store_halfword(ms : &mut MachineState, addr : u32, data :u32) -> Result<(),u32> {
    if (addr&1) != 0 {
        return Err( cp0def::EXCEPT_CODE_ADDR_ERR_STORE );
    }
    return store_memory(ms, addr, 2, data);
}

pub fn store_word(ms : &mut MachineState, addr : u32, data :u32) -> Result<(),u32> {
    if (addr&3) != 0 {
        return Err( cp0def::EXCEPT_CODE_ADDR_ERR_STORE );
    }
    return store_memory(ms, addr, 4, data);
}