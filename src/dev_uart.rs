use log::debug;
use crate::wasm_utils;
use std::io::{stdout, Write};

use std::sync::mpsc::Receiver;

pub struct IoUART{
    pub buffered     : bool,
    pub int_enable   : u8,
    pub int_ident    : u8,
    pub line_control : u8,
    pub modem_control: u8,
    pub divisor      : [u8;2],
    pub buf          : u8,
    pub scratch      : u8,
	pub break_request: bool
}

impl IoUART {
    pub fn new() -> Self {
        Self { 
            buffered: false, 
            int_enable: 0, 
            int_ident: 0, 
            line_control: 0, 
            modem_control: 0, 
            divisor: [0,0], 
            buf: 0, 
            scratch: 0, 
            break_request: false 
        }
    }
}

pub trait UartReadWrite {
    fn read(&mut self)         -> Result<u8,()>;
    fn write(&mut self, _d:char) -> Result<(),()>;
}

pub struct NativeUARTConsole {
    pub receiver: Receiver<u8>
}

impl UartReadWrite for NativeUARTConsole {
    fn read(&mut self)   -> Result<u8,()> {
        return match self.receiver.try_recv() {
            Ok(d) => { Ok(d) }
            _     => { Err(()) }
        }
    }
    fn write(&mut self, data : char)  -> Result<(),()> { 
        print!("{}", data); stdout().flush().unwrap();
        Ok(())
    }
}

pub struct WasmUARTConsole { }

impl UartReadWrite for WasmUARTConsole {
    fn read(&mut self)   -> Result<u8,()> {
        let d = wasm_utils::get_char();
        if d == 0 { Err(()) }else{ Ok(d as u8) }
    }
    fn write(&mut self, data : char)  -> Result<(),()> { 
        wasm_utils::print_string(format!("{}", data));
        Ok(())
    }
}


pub const IOADDR_UART0_BASE  : u32 = 0x18020000;
pub const IOADDR_UART1_BASE  : u32 = 0x18500000;

pub const IOADDR_UART_ADDR_SHIFT : u32 = 2;

pub const IOADDR_UART_SIZE   : u32 = 0x8<<IOADDR_UART_ADDR_SHIFT;
pub const IOADDR_UART_MASK   : u32 = IOADDR_UART_SIZE-1;

pub const UART_REG_RXBUF     : u32 = 0x0<<IOADDR_UART_ADDR_SHIFT; /* read       */
pub const UART_REG_TXBUF     : u32 = 0x0<<IOADDR_UART_ADDR_SHIFT; /* write      */
pub const UART_REG_INTEN     : u32 = 0x1<<IOADDR_UART_ADDR_SHIFT; /* read/write */
pub const UART_REG_INTID     : u32 = 0x2<<IOADDR_UART_ADDR_SHIFT; /* read       */
pub const UART_REG_LINECTL   : u32 = 0x3<<IOADDR_UART_ADDR_SHIFT; /* read/write */
pub const UART_REG_MODEMCTL  : u32 = 0x4<<IOADDR_UART_ADDR_SHIFT; /* read/write */
pub const UART_REG_LINESTAT  : u32 = 0x5<<IOADDR_UART_ADDR_SHIFT; /* read/write */
pub const UART_REG_MODEMSTAT : u32 = 0x6<<IOADDR_UART_ADDR_SHIFT; /* read/write */
pub const UART_REG_SCRATCH   : u32 = 0x7<<IOADDR_UART_ADDR_SHIFT; /* write      */

pub const UART_TX_EMPTY     : u8 = 0x40;
pub const UART_TX_BUF_EMPTY : u8 = 0x20;
pub const UART_RX_BUF_FULL  : u8 = 0x01;

pub const UART_REG_LINECTL_BIT_DLAB : u8 =  7; /* divisor latch access bit */

pub const UART_REG_INTEN_BIT_RX_DATA_AVAIL : u8 = 0;
pub const UART_REG_INTEN_BIT_TX_DATA_EMPTY : u8 = 1;

pub const UART_REG_INTID_NO_INT        : u8 = 1<<0;
pub const UART_REG_INTID_TX_DATA_EMPTY : u8 = 1<<1;
pub const UART_REG_INTID_RX_DATA_AVAIL : u8 = 1<<2;

const ASCII_LF  : u8 = 0x0a;
const ASCII_CR  : u8 = 0x0d;
const ASCII_DEL : u8 = 0x7f;
const ASCII_BS  : u8 = 0x08;


pub fn request_send_break(uart: &mut IoUART){
    uart.break_request = true;
}

pub fn read_reg(uart: &mut IoUART, uart_rw : &mut Box<dyn UartReadWrite>, addr : u32) -> u8 {

    match addr&IOADDR_UART_MASK {
        UART_REG_RXBUF => 
        {
            if 0 != (uart.line_control & (1<<UART_REG_LINECTL_BIT_DLAB)) {
                return uart.divisor[0];
            }else{
                if uart.buffered {
                    uart.buffered = false;
                }

                if uart.buf == ASCII_LF  { uart.buf = ASCII_CR;}
                if uart.buf == ASCII_DEL { uart.buf = ASCII_BS;}

                return uart.buf;
            }
        }
        UART_REG_INTEN => // interrupt enable register
        {
            if 0 != (uart.line_control & (1<<UART_REG_LINECTL_BIT_DLAB)){
                return uart.divisor[1];
            }else{
                return uart.int_enable;
            }
        }
        UART_REG_INTID => // interrupt ident. register
        {
            let prev_id : u8 = uart.int_ident;
            uart.int_ident = UART_REG_INTID_NO_INT;
            return prev_id;
        }
        UART_REG_LINECTL => // line control register
        {
            return uart.line_control;
        }
        UART_REG_MODEMCTL => // modem control register
        {
            return uart.modem_control;
        }
        UART_REG_LINESTAT => // line status register
        {
            // status reg
            // bit0: receive buffer empty if this bit is 0
            // bit1: transmitter idle if this bit is 0

            if ! uart.buffered {
                if uart.break_request {
                    // Send a Ctrl+C
                    uart.buffered = true;
                    uart.buf      = 3;
                    uart.break_request = false; // Clear the request
                }else{
                    match uart_rw.read() {
                        Ok(d) => { uart.buffered = true; uart.buf = d; }
                        _         => { uart.buffered = false; }
                    }
                }
            }

            if uart.buffered {
                if 0!=(uart.int_enable & (1<<UART_REG_INTEN_BIT_RX_DATA_AVAIL)) {
                    uart.int_ident = UART_REG_INTID_RX_DATA_AVAIL;
                }
                return UART_TX_BUF_EMPTY|UART_TX_EMPTY|UART_RX_BUF_FULL;
            }else{
                return UART_TX_BUF_EMPTY|UART_TX_EMPTY;
            }
        }
        UART_REG_MODEMSTAT => // modem status register
        {
            return (1<<7)|(1<<4)|(1<<5); // CTS (Clear To Send) and DSR (Data Set Ready) bits are set
        }

        UART_REG_SCRATCH => // cratch register
        {
            debug!("UART: scratch register was read");
            return uart.scratch;
        }
        _ => {
            ()
        }
    }

    debug!("UART: unknown register was read (addr: 0x{:>x})", addr);

    return 0;
}

pub fn write_reg(uart : &mut IoUART, uart_rw : &mut Box<dyn UartReadWrite>, addr : u32, data : u8){

    match addr&IOADDR_UART_MASK {
        
        UART_REG_TXBUF => 
        {
            if 0 != (uart.line_control & (1<<UART_REG_LINECTL_BIT_DLAB) ){
                uart.divisor[0] = data;
            }else{
                if data == b'\n' {
                    uart_rw.write('\r').unwrap();
                }

                uart_rw.write(data as char).unwrap();

                if 0!=(uart.int_enable & (1<<UART_REG_INTEN_BIT_TX_DATA_EMPTY) ){
                    if uart.int_ident == UART_REG_INTID_NO_INT {
                        uart.int_ident = UART_REG_INTID_TX_DATA_EMPTY;
                    }
                }
            }
        }
        UART_REG_INTEN => // interrupt enable register
        {
            if 0!=( uart.line_control & (1<<UART_REG_LINECTL_BIT_DLAB) ){
                uart.divisor[1] = data;
            }else{
                uart.int_enable = data;
                read_reg(uart, uart_rw, UART_REG_LINESTAT); // to update internal state
            }
        }
        UART_REG_INTID => { () /* read only */ } // interrupt ident. register

        UART_REG_LINECTL => // line control register
        {
            uart.line_control = data;
        }
        UART_REG_MODEMCTL => // modem control register
        {
            uart.modem_control = data;
        }
        UART_REG_LINESTAT => { () } // line status register
        UART_REG_MODEMSTAT => { () } // modem status register
        UART_REG_SCRATCH => // cratch register
        {
            uart.scratch = data;
        }
        _ =>
        {
            debug!("UART: write for unknown register (addr: 0x{:>x}, data 0x{:>x})", addr, data);
        }
    }
}