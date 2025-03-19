use crate::procstate::MachineState;
use crate::dev_uart;


pub const APB_BASE_REG                     :u32 = 0x18000000;

pub const USB_CFG_BASE_REG                 :u32 = APB_BASE_REG + 0x00030000;
pub const GPIO_BASE_REG                    :u32 = APB_BASE_REG + 0x00040000;
pub const PLL_BASE_REG                     :u32 = APB_BASE_REG + 0x00050000;
pub const RST_BASE_REG                     :u32 = APB_BASE_REG + 0x00060000;
pub const GMAC_BASE_REG                    :u32 = APB_BASE_REG + 0x00070000;
pub const RTC_BASE_REG                     :u32 = APB_BASE_REG + 0x00107000;
pub const PLL_SRIF_BASE_REG                :u32 = APB_BASE_REG + 0x00116000;
pub const PCIE_RC0_CTRL_BASE_REG           :u32 = APB_BASE_REG + 0x000F0000;
pub const PCIE_RC1_CTRL_BASE_REG           :u32 = APB_BASE_REG + 0x00280000;

pub const RST_MISC_INTERRUPT_STATUS_REG    :u32 = RST_BASE_REG + 0x10;
pub const RST_MISC_INTERRUPT_MASK_REG      :u32 = RST_BASE_REG + 0x14;
pub const RST_GLOBALINTERRUPT_STATUS_REG   :u32 = RST_BASE_REG + 0x18;
pub const RST_RESET_REG                    :u32 = RST_BASE_REG + 0x1C;

pub const RST_BOOTSTRAP_REG                :u32 = RST_BASE_REG + 0xB0;
pub const RST_REVISION_ID_REG              :u32 = RST_BASE_REG + 0x90;
pub const RST_REVISION_ID_MAJOR_AR9342_VAL :u32 = 0x1120;

pub const PLL_CPU_DDR_CLK_CTRL_REG         :u32 = PLL_BASE_REG + 0x08;

pub const PLL_SRIF_CPU_DPLL_BASE_REG       :u32 = PLL_SRIF_BASE_REG + 0x1C0;
pub const PLL_SRIF_CPU_DPLL1_REG           :u32 = PLL_SRIF_CPU_DPLL_BASE_REG + 0x0;
pub const PLL_SRIF_CPU_DPLL2_REG           :u32 = PLL_SRIF_CPU_DPLL_BASE_REG + 0x4;


pub struct IoMisc{
    pub int_mask      : u32,
    pub reset_request : bool,
}

impl IoMisc {
    pub fn new() -> Self {
        Self { 
            int_mask: 0, 
            reset_request: false,
        }
    }
}

pub struct IoGPIO{
    pub oe  : u32,
    pub out : u32,
}

impl IoGPIO {
    pub fn new() -> Self {
        Self { 
            oe: 0, 
            out: 0,
        }
    }
}

pub fn read_misc_int_status_reg(ms : &MachineState) -> u32{

    let uart_int: bool = 
    0 != ms.uart.int_enable && ms.uart.int_ident != dev_uart::UART_REG_INTID_NO_INT;

    return if uart_int { 1<<3 }else{ 0 };
}
