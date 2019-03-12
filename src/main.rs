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
use cortex_m::peripheral::Peripherals;

use smart_leds::{brightness, Color, SmartLedsWrite};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        // Constrain clocking registers
        //let mut flash = p.FLASH;
        //let mut flash2 = p.FLASH.constrain();
        //let mut rcc = p.RCC.constrain().sysclk(48.mhz()).freeze(&mut flash);
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure pins for SPI
        //let (sck, miso, mosi) = cortex_m::interrupt::free(move |cs| {
        //     (
        //         gpioa.pb3.into_alternate_push_pull(cs),
        //         gpioa.pb4.into_floating_input(cs),
        //         gpioa.pb5.into_alternate_push_pull(cs),
        //     )
        // });

        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        // Configure SPI with 3Mhz rate
        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            apa102::MODE,
            8_000_000.hz(),
            clocks,
            &mut rcc.apb2
        );

        const NUM_LEDS: usize = 200;
        let mut data = [Color::default(); NUM_LEDS];

        let mut apa = Apa102::new(spi);
        let mut up = true;
        loop {
            for j in 0..(256 * 5) {
                for i in 0..NUM_LEDS {
                    data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
                }
                apa.write(brightness(data.iter().cloned(), 255)).unwrap();
                delay.delay_ms(5u8);
            }
        }
    }
    loop {
        continue;
    }
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
