/// FLASHCALW Driver for the SAM4L.


use helpers::*;
use core::mem;

use hil::flash;
use pm;


// Listing of the FLASHCALW register memory map.
// Section 14.10 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct Registers {
    control:                          usize,
    command:                          usize,
    status:                           usize,
    parameter:                        usize,
    version:                          usize,
    general_purpose_fuse_register_hi: usize,
    general_purpose_fuse_register_lo: usize,
}

const FLASHCALW_BASE_ADDRS : *mut Registers = 0x400A0000 as *mut Registers;

#[allow(dead_code)]
enum RegKey {
    CONTROL,
    COMMAND,
    STATUS,
    PARAMETER,
    VERSION,
    GPFRHI,
    GPFRLO
}

// This is the pico cache registers...
// TODO: does this get it's own driver... yea....
/*
struct Picocache_Registers {
    picocache_control:                      usize,
    picocache_status:                       usize,
    picocache_maintenance_register_0:       usize,
    picocache_maintenance_register_1:       usize,
    picocache_montior_configuration:        usize,
    picocache_monitor_enable:               usize,
    picocache_monitor_control:              usize,
    picocache_monitor_status:               usize,
    version:                                usize
}
*/

// There are 18 recognized commands possible to command the flash
// Table 14-5.
#[derive(Clone, Copy)]
pub enum FlashCMD {
    NOP,
    WP,
    EP,
    CPB,
    LP,
    UP,
    EA,
    WGPB,
    EGPB,
    SSB,
    PGPFB,
    EAGPF,
    QPR,
    WUP,
    EUP,
    QPRUP,
    HSEN,
    HSDIS,
}

//The two Flash speeds
#[derive(Clone, Copy)]
pub enum Speed {
    Standard,
    HighSpeed
}

// The FLASHCALW controller
//TODO: finishing beefing up...
pub struct FLASHCALW {
    registers: *mut Registers,
    // might make these more specific
    ahb_clock: pm::Clock,
    hramc1_clock: pm::Clock,
    pb_clock: pm::Clock,
    speed_mode: Speed,
    wait_until_ready : fn(&FLASHCALW) -> (),
    error_status : u32
    //client: TakeCell...

}

//static instance for the board. Only one FLASHCALW on chip.
pub static mut flash_controller : FLASHCALW = 
    FLASHCALW::new(FLASHCALW_BASE_ADDRS, pm::HSBClock::FLASHCALW, 
        pm::HSBClock::FLASHCALW, pm::PBBClock::FLASHCALW, Speed::Standard);


// Few constants relating to module configuration.
const FLASH_PAGE_SIZE : u32 = 512;
const FLASH_NB_OF_REGIONS : u32 = 16;
const FLASHCALW_REGIONS : u32 = FLASH_NB_OF_REGIONS;
const FLASHCALW_CMD_KEY : usize = 0xA5;

//Various operating frequencies
const FLASH_FREQ_PS1_FWS_1_FWU_MAX_FREQ : u32 = 12000000;
const FLASH_FREQ_PS0_FWS_0_MAX_FREQ : u32 = 18000000;
const FLASH_FREQ_PS0_FWS_1_MAX_FREQ : u32 = 36000000;
const FLASH_FREQ_PS1_FWS_0_MAX_FREQ : u32 = 8000000;

// These frequencies is not used anywhere, but are in the original library...
// so commenting them out...
// const FLASH_FREQ_PS1_FWS_1_MAX_FREQ : u32 = 12000000; 
//const FLASH_FREQ_PS2_FWS_1_MAX_FREQ : u32 = 48000000;

#[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_ENABLE)]
const FLASH_FREQ_PS2_FWS_0_MAX_FREQ : u32 = 24000000;

//helper for gp fuses all one...
const GP_ALL_FUSES_ONE : u64 = !0 as u64;

// TODO: should export this to a chip specific module or so... something that gives me size.
//const FLASHCALW_SIZE : usize = 512; // instead I'll just read it straight from the table
                                      // which will be alloced only for a fxn call.

macro_rules! get_bit {
    ($w:expr) => (0x1u32 << $w);
}

// save me some casts...
macro_rules! get_ubit {
    ($w:expr) => (0x1usize << $w);
}

/// This is simply std::cmp::min from std
#[inline]
fn min<T: Ord>(v1: T, v2: T) -> T {
    if v1 <= v2 { v1 } else { v2 }    
}

pub fn default_wait_until_ready(flash : &FLASHCALW) {
    while !flash.is_ready(){}    
}



impl FLASHCALW {
    //#![feature(const_fn)]
    const fn new(base_addr: *mut Registers, ahb_clk: pm::HSBClock,
    hramc1_clk: pm::HSBClock, pb_clk: pm::PBBClock, mode : Speed) -> FLASHCALW {
        FLASHCALW {
            registers: base_addr,
            ahb_clock: pm::Clock::HSB(ahb_clk),
            hramc1_clock: pm::Clock::HSB(hramc1_clk),
            pb_clock: pm::Clock::PBB(pb_clk),
            speed_mode: mode,
            wait_until_ready: default_wait_until_ready,
            error_status: 0
        }
    }

    //simple helper to read the register (use if only one call needs to be accessed 
    // your fxn.
    fn read_register(&self, key : RegKey) -> usize {
        let registers : &mut Registers = unsafe { 
            mem::transmute(self.registers)
        };
        
        match key {
            RegKey::CONTROL => {
                volatile_load(&registers.control)    
            },
            RegKey::COMMAND => {
                volatile_load(&registers.command)    
            },
            RegKey::STATUS => {
                volatile_load(&registers.status)
            },
            RegKey::PARAMETER => {
                volatile_load(&registers.parameter)
            },
            RegKey::VERSION => {
                volatile_load(&registers.version)
            },
            RegKey::GPFRHI => {
                volatile_load(&registers.general_purpose_fuse_register_hi)
            },
            RegKey::GPFRLO => {
                volatile_load(&registers.general_purpose_fuse_register_lo)
            }
        }
    }

    pub fn handle_interrupt(&self) {
        use hil::flash::Error;
        
        let status = self.read_register(RegKey::STATUS);
        //the status register is now automatically cleared...

        /*
        let err = match status {
            x if x & (1 <<     
        };*/

        //TODO: implement...
        

    }
  
  
    /// FLASH properties.
    pub fn get_flash_size(&self) -> u32 {
        let flash_sizes = [4,
                           8,
                           16,
                           32,
                           48,
                           64,
                           96,
                           128,
                           192,
                           256,
                           384,
                           512,
                           768,
                           1024,
                           2048];
        flash_sizes[self.read_register(RegKey::PARAMETER) & 0xf] // get the FSZ number and 
                                                    // lookup in the table for the size.
    }

    pub fn get_page_count(&self) -> u32 {
        self.get_flash_size() / FLASH_PAGE_SIZE    
    }

    pub fn get_page_count_per_region(&self) -> u32 {
        self.get_page_count() / FLASH_NB_OF_REGIONS
    }


    pub fn get_page_region(&self, page_number : i32) -> u32 {
        (if page_number >= 0 
            { unsafe { mem::transmute(page_number) } } 
        else 
            { self.get_page_number() } 
        / self.get_page_count_per_region())
    }

    pub fn get_region_first_page_number(&self, region : u32) -> u32 {
        region * self.get_page_count_per_region()    
    }


    /// FLASHC Control
    fn change_control_single_bit_val(&self, position : u32, enable : bool) {
       let regs : &mut Registers = unsafe { mem::transmute(self.registers)};
       let mut reg_val = volatile_load(&regs.control) & !get_ubit!(position);
       if enable {
            reg_val |= get_ubit!(position); 
       }
        
       volatile_store(&mut regs.control, reg_val);
    }

    pub fn get_wait_state(&self) -> u32 {
        if self.read_register(RegKey::CONTROL) & get_ubit!(6) == 0 {
            0
        } else{
            1
        }
    }

    pub fn set_wait_state(&self, wait_state : u32) {
        if wait_state == 1 {
            self.change_control_single_bit_val(6, true);
        } else {
            self.change_control_single_bit_val(6, false);
        }
    }
    
    //depending on if this flag is passed in this function is implemented differently.
    #[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_ENABLE)]
    pub fn set_flash_waitstate_and_readmode(&mut self, cpu_freq : u32, 
        _ps_val : u32, _is_fwu_enabled : bool) {
        //ps_val and is_fwu_enabled not used in this implementation.
        if cpu_freq > FLASH_FREQ_PS2_FWS_0_MAX_FREQ {
            self.set_wait_state(1);    
        } else {
            self.set_wait_state(0);
        }

        self.issue_command(FlashCMD::HSEN, -1);
    }

    #[cfg(not(CONFIG_FLASH_READ_MODE_HIGH_SPEED_ENABLE))]
    pub fn set_flash_waitstate_and_readmode(&mut self, cpu_freq : u32, 
        ps_val : u32, is_fwu_enabled : bool) {
        if ps_val == 0 {
            if cpu_freq > FLASH_FREQ_PS0_FWS_0_MAX_FREQ {
                self.set_wait_state(1);
                if cpu_freq <= FLASH_FREQ_PS0_FWS_1_MAX_FREQ {
                    self.issue_command(FlashCMD::HSDIS, -1);
                } else {
                    self.issue_command(FlashCMD::HSEN, -1);
                }
            }else {
                if is_fwu_enabled && cpu_freq <= FLASH_FREQ_PS1_FWS_1_FWU_MAX_FREQ {
                    self.set_wait_state(1);
                    self.issue_command(FlashCMD::HSDIS, -1);
                } else {
                    self.set_wait_state(0);
                    self.issue_command(FlashCMD::HSDIS, -1);
                }
            }
        
        } else {
            /* ps_val == 1 */
            if cpu_freq > FLASH_FREQ_PS1_FWS_0_MAX_FREQ {
                self.set_wait_state(1);    
            } else {
                self.set_wait_state(0);
            }
            self.issue_command(FlashCMD::HSDIS, -1);
        }
    }


    pub fn is_ready_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & get_ubit!(0)) != 0
    }

    pub fn enable_ready_int(&self, enable : bool) {
        self.change_control_single_bit_val(0, enable); 
    }

    pub fn is_lock_error_int_enabled(&self) -> bool {
        (self.read_register(RegKey::COMMAND) & get_ubit!(2)) != 0
    }

    pub fn enable_lock_error_int(&self, enable : bool) {
        self.change_control_single_bit_val(2, enable);
    }

    pub fn is_prog_error_int_enabled(&self) -> bool {
        (self.read_register(RegKey::COMMAND) & get_ubit!(3)) != 0
    }

    pub fn enable_prog_error_int(&self, enable : bool) {
       self.change_control_single_bit_val(3, enable);
    }

    ///Flashcalw status

    pub fn is_ready(&self) -> bool {
        self.read_register(RegKey::STATUS) & get_ubit!(0) != 0
    }

    pub fn get_error_status(&self) -> u32 {
        (self.read_register(RegKey::STATUS) as u32) & ( get_bit!(3) | get_bit!(2))    
    }

    pub fn is_lock_error(&self) -> bool {
        self.read_register(RegKey::STATUS) & get_ubit!(2) != 0
    }

    pub fn is_programming_error(&self) -> bool {
        self.read_register(RegKey::STATUS) & get_ubit!(3) != 0    
    }

    ///Flashcalw command control
    pub fn get_command(&self) -> u32 {
        ((self.read_register(RegKey::COMMAND) as u32) & (get_bit!(6) - 1))
    }

    pub fn get_page_number(&self) -> u32 {
        //create a mask for the page number field
        let mut page_mask : usize = get_ubit!(8) - 1;
        page_mask |= page_mask << 24;
        page_mask = !page_mask;

        ((self.read_register(RegKey::COMMAND) & page_mask) >> 8) as u32
    }
    
    pub fn issue_command(&mut self, command : FlashCMD, page_number : i32) {
        (self.wait_until_ready)(self); // call the registered wait function
        let cmd_regs : &mut Registers = unsafe {mem::transmute(self.registers)};
        let mut reg_val : usize = volatile_load(&mut cmd_regs.command);
        
        let clear_cmd_mask : usize = (!(get_bit!(6) - 1)) as usize;
        reg_val &= clear_cmd_mask;
        
        // craft the command
        if page_number >= 0 {
            reg_val =   FLASHCALW_CMD_KEY << 24     | 
                        (page_number as usize) << 8   |
                        command as usize;
        } else {
            reg_val |= FLASHCALW_CMD_KEY << 24 | command as usize;     
        }
        volatile_store(&mut cmd_regs.command, reg_val); // write the cmd

        self.error_status = self.get_error_status();
        (self.wait_until_ready)(self);
    }


    /// Flashcalw global commands
    pub fn flashcalw_no_operation(&mut self) {
        self.issue_command(FlashCMD::NOP, -1);        
    }

    pub fn erase_all(&mut self) {
        self.issue_command(FlashCMD::EA, -1);    
    }

    ///FLASHCALW Protection Mechanisms
    pub fn is_security_bit_active(&self) -> bool {
        (self.read_register(RegKey::STATUS) & get_ubit!(4)) != 0
    }

    pub fn set_security_bit(&mut self) {
        self.issue_command(FlashCMD::SSB, -1);
    }

    pub fn is_page_region_locked(&self, page_number : u32) -> bool {
        self.is_region_locked(self.get_page_region(page_number as i32))
    }

    pub fn is_region_locked(&self, region : u32) -> bool {
        (self.read_register(RegKey::STATUS) & get_ubit!(region + 16)) != 0    
    }
    
    pub fn lock_page_region(&mut self, page_number : i32, lock : bool) {
        if lock {
            self.issue_command(FlashCMD::LP, page_number);
        } else {
            self.issue_command(FlashCMD::UP, page_number);
        }
    }

    pub fn lock_region(&mut self, region : u32, lock : bool) {
        let first_page : i32 = self.get_region_first_page_number(region) as i32;
        self.lock_page_region(first_page, lock);    
    }

    pub fn lock_all_regions(&mut self, lock : bool) {
        let mut error_status = 0;
        let mut num_regions = 0;

        while num_regions < FLASHCALW_REGIONS {
            self.lock_region(num_regions, lock);
            error_status |= self.error_status;    
            num_regions += 1;
        }
        
        self.error_status = error_status;
    }

    ///Flashcalw General-Purpose Fuses
    pub fn read_gp_fuse_bit(&self, gp_fuse_bit : u32) -> bool {
        (self.read_all_gp_fuses() & (1u64 << (gp_fuse_bit & 0x3F))) != 0    
    }

    pub fn read_gp_fuse_bitfield(&self, pos : u32, width : u32) -> u64 {
        self.read_all_gp_fuses() >> (pos & 0x3F) & 
            ((1u64 << min(width, 64)) - 1)
    }

    pub fn read_gp_fuse_byte(&self, gp_fuse_byte : u32) -> u8 {
        (self.read_all_gp_fuses() >> ((gp_fuse_byte & 0x07) << 3)) as u8
    }

    pub fn read_all_gp_fuses(&self) -> u64 {
        let registers : &mut Registers = unsafe {
            mem::transmute(self.registers)  
        };
        let gpfrhi : u64 = volatile_load(&registers.general_purpose_fuse_register_hi) as u64;
        let gpfrlo : u64 = volatile_load(&registers.general_purpose_fuse_register_lo) as u64;
        (gpfrlo | (gpfrhi << 32))
    }
    
    pub fn erase_gp_fuse_bit(&mut self, gp_fuse_bit : u32, check : bool) -> bool {
        self.issue_command(FlashCMD::EGPB, (gp_fuse_bit as i32) & 0x3F);
        if check {
            self.read_gp_fuse_bit(gp_fuse_bit)
        } else {
            true    
        }
    }

    pub fn erase_gp_fuse_bitfield(&mut self, mut pos : u32, mut width : u32, check : bool) -> bool {
        let mut error_status : u32 = 0;

        pos &= 0x3F;
        width = min(width, 64);
        for gp_fuse_bit in pos..(pos + width) {
            self.erase_gp_fuse_bit(gp_fuse_bit, false);
            error_status |= self.error_status;
        }

        self.error_status = error_status;
        if check {
            self.read_gp_fuse_bitfield(pos, width) == ((1u64 << width) - 1)
        } else {
            true
        }
    }

    pub fn erase_gp_fuse_byte(&mut self, gp_fuse_byte : u32, check : bool) -> bool {
        let mut error_status : u32;
        let mut value = self.read_all_gp_fuses();
        let mut byte_val : u8;

        self.erase_all_gp_fuses(false);
        error_status = self.error_status;

        for current_gp_fuse_byte in 0..8 {
            if current_gp_fuse_byte != gp_fuse_byte {
                byte_val = (value & ((1u64 << 8) - 1)) as u8;
                self.write_gp_fuse_byte(current_gp_fuse_byte, byte_val);
                error_status |= self.error_status;
            }
            value >>= 8;    
        }

        self.error_status = error_status;
        
        if check {
            self.read_gp_fuse_byte(gp_fuse_byte) == 0xFF
        } else {
            true    
        }
    }

    pub fn erase_all_gp_fuses(&mut self, check : bool) -> bool {
        self.issue_command(FlashCMD::EAGPF, -1);
        if check {
            self.read_all_gp_fuses() == (GP_ALL_FUSES_ONE)
        } else {
            true
        }
    }

    pub fn write_gp_fuse_bit(&mut self, gp_fuse_bit : u32, value : bool) {
        if !value {
            self.issue_command(FlashCMD::WGPB, (gp_fuse_bit as i32) & 0x3F)
        }    
    }

    pub fn write_gp_fuse_bitfield(&mut self, mut pos : u32, mut width : u32, mut value : u64) {
        let mut error_status : u32 = 0;

        pos &= 0x3F;
        width = min(width, 64);

        for gp_fuse_bit in pos..(pos + width) {
            self.write_gp_fuse_bit(gp_fuse_bit, value & 0x01 != 0);
            error_status |= self.error_status;
            value >>= 1;
        }
        self.error_status = error_status;
    }

    pub fn write_gp_fuse_byte(&mut self, gp_fuse_byte : u32, value : u8) {
        self.issue_command(FlashCMD::PGPFB, ((gp_fuse_byte & 0x07) | (value as u32) << 3) as i32);    
    }

    pub fn write_all_gp_fuses(&mut self, mut value : u64) {
        let mut error_status : u32 = 0;
        let mut byte_val : u8;

        for gp_fuse_byte in 0..8 {
            //get the lower byte
            byte_val = (value & ((1u64 << 8) - 1)) as u8;
            self.write_gp_fuse_byte(gp_fuse_byte, byte_val);
            error_status |= self.error_status;
            value >>= 8;
        }
            self.error_status = error_status;
    }

    pub fn set_gp_fuse_bit(&mut self, gp_fuse_bit : u32, value : bool) {
        if value {
            self.erase_gp_fuse_bit(gp_fuse_bit, false);    
        } else {
            self.write_gp_fuse_bit(gp_fuse_bit, false);    
        }
    }

    pub fn set_gp_fuse_bitfield(&mut self, mut pos: u32, mut width : u32, mut value : u64) {
        let mut error_status : u32 = 0;

        pos &= 0x3F;
        width = min(width, 64);

        for gp_fuse_bit in pos..(pos + width) {
            self.set_gp_fuse_bit(gp_fuse_bit, value & 0x01 != 0);
            error_status |= self.error_status;
            value >>= 1;
        }
        self.error_status = error_status;
    }

    pub fn set_gp_fuse_byte(&mut self, gp_fuse_byte : u32, value : u8) {
        let error_status : u32;
        match value {
            0xFF => {
                self.erase_gp_fuse_byte(gp_fuse_byte, false);    
            },
            0x00 => {
                self.write_gp_fuse_byte(gp_fuse_byte, 0x00);
            },
            _ => {
                self.erase_gp_fuse_byte(gp_fuse_byte, false);
                error_status = self.error_status;
                self.write_gp_fuse_byte(gp_fuse_byte, value);
                self.error_status |= error_status;
            }
        };

    }

    pub fn set_all_gp_fuses(&mut self, value : u64) {
        let error_status : u32;

        match value {
            GP_ALL_FUSES_ONE => {
                self.erase_all_gp_fuses(false);  
            },
            0u64 => {
                self.write_all_gp_fuses(0u64);  
            },
            _ => {
                self.erase_all_gp_fuses(false);
                error_status = self.error_status;
                self.write_all_gp_fuses(value);
                self.error_status |= error_status;
            }
        }
    }
    
    ///Flashcalw Access to Flash Pages
    pub fn clear_page_buffer(&mut self) {
        self.issue_command(FlashCMD::CPB, -1)    
    }

    pub fn is_page_erased(&self) -> bool {
        let registers : &mut Registers = unsafe {
            mem::transmute(self.registers)     
        };
        let status = volatile_load(&registers.status);

        (status & get_ubit!(5)) != 0    
    }

    pub fn quick_page_read(&mut self, page_number : i32) -> bool {
        self.issue_command(FlashCMD::QPR, page_number);
        self.is_page_erased()
    }

    pub fn erase_page(&mut self, page_number : i32, check : bool) -> bool {
        let mut page_erased = true;

        self.issue_command(FlashCMD::EP, page_number);
        if check {
            let error_status : u32 = self.error_status;
            page_erased = self.quick_page_read(-1);
            self.error_status |= error_status;
        }

        page_erased
    }

    pub fn erase_all_pages(&mut self, check : bool) -> bool {
        let mut all_pages_erased = true;
        let mut error_status : u32 = 0;
        let mut page_number : i32 = (self.get_page_count() as i32) - 1;

        while page_number >= 0 {
            all_pages_erased &= self.erase_page(page_number, check);
            error_status |= self.error_status;
            page_number -= 1;
        }
        self.error_status = error_status;
        all_pages_erased
    }

    pub fn write_page(&mut self, page_number : i32) {
        self.issue_command(FlashCMD::WP, page_number);    
    }

    pub fn quick_user_page_read(&mut self) -> bool {
        self.issue_command(FlashCMD::QPRUP, -1);
        self.is_page_erased()
    }

    pub fn erase_user_page(&mut self, check : bool) -> bool {
        self.issue_command(FlashCMD::EUP, -1);    
        if check {
            self.quick_user_page_read()
        } else {
            true    
        }
    }

    pub fn write_user_page(&mut self) {
        self.issue_command(FlashCMD::WUP, -1);    
    } 
    
    //TODO: implement memset / memcpy fxns.
    
}

// implement the generic calls using the low-lv functions.
impl flash::FlashController for FLASHCALW {
    
    fn configure(&self) {
        unimplemented!()    
    }

    fn get_page_size(&self) -> u32 {
        FLASH_PAGE_SIZE
    }

    fn get_flash_size(&self) -> u32 {
        //check clock
        self.get_flash_size()
    }

    fn read_page(&self, addr: usize, data: &'static mut [u8], len: u8) {
        unimplemented!()    
    }
    
    fn write_page(&self, addr: usize, data: &'static mut [u8], len: u8) {
        unimplemented!()    
    }
    
    fn erase_page(&self, page_num: usize) {
        unimplemented!()    
    }

    fn current_page(&self) -> i32 {
        unimplemented!()    
    }
}

