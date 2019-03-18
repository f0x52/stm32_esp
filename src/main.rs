#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use apa102_spi as apa102;
use stm32f1xx_hal as hal;

use crate::apa102::Apa102;
use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::stm32;
use crate::hal::serial::Serial;
use crate::hal::serial::Tx;
use crate::hal::serial::Rx;
use crate::hal::pac::USART2;
use cortex_m::peripheral::Peripherals;
use nb::block;

use core::fmt;
use core::fmt::Write;
use core::str;

use smart_leds::{brightness, Color, SmartLedsWrite};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);
        let mut delay = Delay::new(cp.SYST, clocks);

        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx = gpioa.pa3;

        let serial = Serial::usart2(
            p.USART2,
            (tx, rx),
            &mut afio.mapr,
            115200.bps(),
            clocks,
            &mut rcc.apb1,
        );

        let (mut tx, mut rx) = serial.split();

        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            apa102::MODE,
            8_000_000.hz(),
            clocks,
            &mut rcc.apb2
        );

        const NUM_LEDS: usize = 21;
        let mut data = [Color::default(); NUM_LEDS];

        let mut apa = Apa102::new(spi);
        for i in 0..NUM_LEDS {
            data[i] = (20, 0, 0).into();
        }
        apa.write(brightness(data.iter().cloned(), 255)).unwrap();
        write!(tx, "AT");
        loop {
            let mut received = [0;2];
            rx = read(rx, &mut received).unwrap();
            unsafe {
                let a = str::from_utf8_unchecked(&received);
                for i in 0..NUM_LEDS {
                    data[i] = (0, 50, 0).into();
                }
                apa.write(brightness(data.iter().cloned(), 255)).unwrap();
                //writeln!(tx, "ECHO: {}", a);
                delay.delay_ms(100u8);
                write!(tx, "\n\n\ntest123");
            }
        }

        loop {
            let received = block!(rx.read()).unwrap();
            if (received == b'O') {
                for i in 0..NUM_LEDS {
                    data[i] = (0, 255, 0).into();
                }
                apa.write(brightness(data.iter().cloned(), 255)).unwrap();
            }
        }

        loop {
            // for i in 0..NUM_LEDS {
            //     data[i] = (40, 20, 0).into();
            // }
            //apa.write(brightness(data.iter().cloned(), 30)).unwrap();

            // for j in 0..(256 * 5) {
            //     for i in 0..NUM_LEDS {
            //         data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            //     }
            //     apa.write(brightness(data.iter().cloned(), 255)).unwrap();
            //     delay.delay_ms(5u8);
            // }
        }
    }
    loop {
        continue;
    }
}

fn read(mut rx: Rx<USART2>, buf: &mut [u8]) -> Result<Rx<USART2>, ()> {
    let length = buf.len();
    for i in 0..length {
        let byte = block!(rx.read()).unwrap();
        buf[i] = byte;
    }
    Ok(rx)
}

fn wheel(mut wheel_pos: u8) -> Color {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
