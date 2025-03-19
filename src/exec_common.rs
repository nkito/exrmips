
#[macro_export]
macro_rules! sign_ext16  { ( $x:expr ) => (( ($x as i16) as i32 ) as u32 ) }
#[macro_export]
macro_rules! zero_ext16  { ( $x:expr ) => (  ($x as u16) as u32 ) }
#[macro_export]
macro_rules! sign_ext8   { ( $x:expr ) => (( ($x as i8 ) as i32 ) as u32 ) }
#[macro_export]
macro_rules! zero_ext8   { ( $x:expr ) => (  ($x as u8 ) as u32 ) }

#[macro_export]
macro_rules! sign_ext15  { ( $x:expr ) => ((((((($x<<1) as i16) as i32 ) >> 1) as i16) as i32 ) as u32 ) }
#[macro_export]
macro_rules! sign_ext11  { ( $x:expr ) => ((((((($x<<5) as i16) as i32 ) >> 5) as i16) as i32 ) as u32 ) }

#[macro_export]
macro_rules! sign_ext4   { ( $x:expr ) => ((((((($x<<4) as  i8) as i32 ) >> 4) as i16) as i32 ) as u32 ) }
#[macro_export]
macro_rules! zero_ext4   { ( $x:expr ) => (  (($x & 0xf) as u8 ) as u32 ) }

#[macro_export]
macro_rules! update_pc_next32 { ( $ms:expr ) => ( 
    {
        if( $ms.reg.delay_en ){
            $ms.reg.pc = $ms.reg.pc_delay;
            $ms.reg.delay_en = false; 
        }else{     
            $ms.reg.pc += 4;
        }  
    } ) }

#[macro_export]
macro_rules! update_pc_next16 { ( $ms:expr ) => ( 
    {
        if( $ms.reg.delay_en ){
            $ms.reg.pc = $ms.reg.pc_delay;
            $ms.reg.delay_en = false; 
        }else{     
            $ms.reg.pc += 2;
        }  
    } ) }


#[macro_export]
macro_rules! update_pc_next32_with_delayed_imm { ( $ms:expr , $imm:expr ) => ( 
    {
        $ms.reg.pc_delay = $imm;
        $ms.reg.pc_prev_jump = $ms.reg.pc;
        $ms.reg.pc += 4;
        $ms.reg.delay_en = true;
    } ) }

#[macro_export]
macro_rules! update_pc_next16_with_delayed_imm { ( $ms:expr , $imm:expr ) => ( 
    {
        $ms.reg.pc_delay = $imm;
        $ms.reg.pc_prev_jump = $ms.reg.pc;
        $ms.reg.pc += 2;
        $ms.reg.delay_en = true;
    } ) }

#[macro_export]
macro_rules! update_pc_imm { ( $ms:expr , $imm:expr ) => ( 
    {
        $ms.reg.pc_prev_jump = $ms.reg.pc;
        $ms.reg.pc = $imm;
        $ms.reg.delay_en = false;
    } ) }

