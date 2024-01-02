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

fn busy_wait(flash: &FLASH) {
    while flash.sr.read().bsy().bit_is_set() {}
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut flash = dp.FLASH;
    let page_addr = 0x0800_1800;

    unsafe {
        flash.keyr.write(|w| w.bits(UNLOCK_KEY1));
        flash.keyr.write(|w| w.bits(UNLOCK_KEY2));
        flash.cr.write(|w| w.per().set_bit().strt().set_bit());

        busy_wait(&flash);

        flash.ar.write(|w| w.bits(page_addr));
        flash
            .cr
            .write(|w| w.per().set_bit().strt().set_bit().pg().set_bit());

        busy_wait(&flash);

        if flash.sr.read().eop().bit_is_set() {
            flash.sr.modify(|_, w| w.eop().clear_bit());
            flash.cr.write(|w| w.per().clear_bit());
        } else {
            hprintln!("ERROR -> PAGE WAS NOT ERASED..").unwrap();
            let pgerr = flash.sr.read().pgerr().bit_is_set();
            if pgerr {
                hprintln!("FLASH PROGRAMMING SEQ ERROR").unwrap();
            }

            let wrptrerr = flash.sr.read().wrprterr().bit_is_set();
            if wrptrerr {
                hprintln!("FLASH PROGRAMMING WRITE PROTECTION ERROR").unwrap();
            }
        }

        hprintln!("PAGE ERASE PART DONE.").unwrap();

        // Reset status and control registers.
        flash.sr.write(|w| w.bits(0x0));
        flash.cr.write(|w| w.bits(0x0));

        busy_wait(&flash);
        flash.cr.modify(|_, w| w.bits(0).pg().set_bit());
        ptr::write_volatile(page_addr as *mut u16, 0xffff);

        busy_wait(&flash);

        let status = flash.sr.read().pgerr();
        if status.bit_is_set() {
            hprintln!("PROGRAMMING SEQ ERROR").unwrap();
        }

        let status = flash.sr.read().wrprterr();
        if status.bit_is_set() {
            hprintln!("WRITE PROTECTION ERROR").unwrap();
        }

        let read_value = ptr::read_volatile(page_addr as *const u16) as u16;
        hprintln!("READ VALUE IS: {}", read_value);
    }

    loop {}
}
