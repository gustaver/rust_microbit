#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal_nb::serial::{Read, Write};
use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprintln};
use heapless::Vec;
use core::fmt::Write as CoreWrite;

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

mod serial_setup;
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 32> = Vec::new();

    loop {
        buffer.clear();

        loop {
            // We assume that the receiving cannot fail
            let byte = nb::block!(serial.read()).unwrap();
            nb::block!(serial.write(byte)).unwrap();
            nb::block!(serial.flush()).unwrap();

            if buffer.push(byte).is_err() {
                write!(serial, "error: buffer full\r\n").unwrap();
                break;
            }

            if byte == 13 {
                nb::block!(serial.write(b'\n')).unwrap();
                nb::block!(serial.write(b'\r')).unwrap();
                nb::block!(serial.flush()).unwrap();
                for &byte in buffer.iter().rev().chain(&[b'\n', b'\r']) {
                    nb::block!(serial.write(byte)).unwrap();
                }
                break;
            }
        }
        nb::block!(serial.flush()).unwrap()
    }
}
