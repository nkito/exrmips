use crate::cp0def;

pub struct AddrCache{
    vaddr : u32,  /* virtual address */
    ppage : u32,  /* [31:12] phy page address */
    asid  : u8,   /* asid */
    valid : bool,
    mode  : u8,  /* [4:3] mode, [2] Error Level, [1] Exception Level */
}

impl AddrCache {
    pub fn new() -> Self {
        Self { vaddr: 0, ppage: 0, asid: 0, valid:false, mode:0 }
    }

    pub fn clear(self : &mut AddrCache){
        self.valid = false;
    }
    
    pub fn check(self : &AddrCache, vaddr : u32, asid  : u32, mode : u32 ) -> bool {
        let mask = !(0xfff as u32);
    
        if ! self.valid { return false; }
    
        if mode as u8 != self.mode { return false; }
    
        if (vaddr & mask) == self.vaddr {
            if asid as u8 == self.asid {
                return true;
            }
        }
        return false;
    }

    pub fn get_addr(self : &AddrCache, vaddr : u32) -> u32 {
        return self.ppage | (vaddr & 0xfff);
    }
    
    pub fn set(self : &mut AddrCache, vaddr : u32, asid : u32, mode : u32, paddr : u32){
        let mask = !(0xfff as u32);
    
        self.ppage = paddr & mask;
        self.asid  = (asid & cp0def::C0_ENTRYHI_ASID_MASK) as u8;
        self.mode  = mode as u8;
        self.vaddr = vaddr & mask;
        self.valid = true;
    }
}
