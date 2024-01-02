#![allow(unsafe_code, unused)]
#![no_main]
#![no_std]

use core::borrow::BorrowMut;
use core::ptr;

use cortex_m::asm::nop;
use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use stm32f3xx_hal_v2::{flash::ACR, pac::Peripherals, pac::FLASH};

const UNLOCK_KEY1: u32 = 0x45670123;
const UNLOCK_KEY2: u32 = 0xCDEF89AB;

fn wait_ready(flash: &FLASH) {
    while flash.sr.read().bsy().bit() {}
}

fn wipe_page(flash: &FLASH) {
    wait_ready(flash);

    unsafe {
        flash.cr.write(|w| w.per().set_bit());
        flash.ar.write(|w| w.bits(0x0800_1800));
        flash.cr.write(|w| w.strt().set_bit());
    }

    wait_ready(flash);

    if flash.sr.read().eop().bit_is_set() {
        hprintln!("SOME FLASH OPERATION DONE> CLEAR");
    } else {
        hprintln!("NO FLASH OPERATION DONE> CLEAR");
    }

    flash.cr.write(|w| w.per().clear_bit());
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut flash = dp.FLASH;

    let addr = 0x0800_1800;

    wait_ready(&flash);

    // Unlock the flash
    flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEY1) });
    flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEY2) });

    wipe_page(&flash);
    
    unsafe {
        if flash.cr.read().lock().bit_is_set() {
            hprintln!(" =>>>>>>>>>>>>>>>>>>>>>>>> Flash memory is locked.");
        }

        // Clear out the status register.
        flash.sr.write(|w| w.bits(0x0));
        flash.cr.write(|w| w.pg().set_bit().strt().set_bit());
        // flash.cr.modify(|k, w| unsafe { w.bits(0) });

        let data = 0x12;
        flash.ar.write(|w| w.bits(addr));
        ptr::write_volatile((addr as *mut u16), data);
    }

    wait_ready(&flash);

    let bytes = [1u8, 2u8, 3u8];

    flash.cr.modify(|_, w| w.pg().clear_bit());

    // Trivial errors handling.
    let status = flash.sr.read();
    let write_protection_err = status.wrprterr().bit_is_set();
    let programming_seq_err = status.pgerr().bit_is_set();
    let flash_op_done = status.eop().bit_is_set();
    
    if flash_op_done {
        hprintln!("FLASH OPERATION DONE!").unwrap();
    } else {
        hprintln!("FLASH OPERATION NOT DONE!").unwrap();
    }
    
    if write_protection_err {
        hprintln!("WRITE PROTECTION ERROR!").unwrap();
    }
    if programming_seq_err {
        hprintln!("PROGRAMMING SEQ ERROR!").unwrap();
    }

    flash.sr.write(|w| w.eop().clear_bit());
    flash.cr.write(|w| w.pg().clear_bit());

    // Lock the flash.
    flash.cr.modify(|_, w| w.lock().set_bit());

    unsafe {
        let val = ptr::read_volatile(addr as *mut u32);
        hprintln!("Written value is {}", val).unwrap();
    }

    loop {}
}

