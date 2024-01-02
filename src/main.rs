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

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut flash = dp.FLASH;

    // Unlock the flash
    flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEY1) });
    flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEY2) });

    wait_ready(&flash);

    let addr = 0x0800_1800 as u32;


    unsafe {
        if flash.cr.read().lock().bit_is_set() {
            hprintln!(" =>>>>>>>>>>>>>>>>>>>>>>>> Flash memory is locked.");
        }
        flash.cr.write(|w| w.bits(0x0));
        flash.cr.write(|w| w.pg().set_bit());
        // flash.cr.modify(|k, w| unsafe { w.bits(0) });

        let data = 13u16;
        flash.ar.write(|w| w.bits(addr));
        ptr::write_volatile(addr as *mut u16, data);
    }

    wait_ready(&flash);

    flash.cr.write(|w| w.pg().clear_bit());

    let status = flash.sr.read();

    // Lock the flash.
    // flash.cr.modify(|_, w| w.lock().set_bit());

    hprintln!("Hello, world!").unwrap();
    loop {}
}
