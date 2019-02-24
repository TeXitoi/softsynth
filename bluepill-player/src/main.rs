#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use softsynth::{Oscillator, RATE};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{pwm, stm32};

struct PwmSoundCard<Pwm1, Pwm2> {
    pwm1: Pwm1,
    pwm2: Pwm2,
}
impl<Pwm1, Pwm2> PwmSoundCard<Pwm1, Pwm2>
where
    Pwm1: embedded_hal::PwmPin,
    Pwm1::Duty: From<u16> + Into<u32>,
    Pwm2: embedded_hal::PwmPin,
    Pwm2::Duty: From<u16> + Into<u32>,
{
    fn set(&mut self, val: i16) {
        if val > 0 {
            let max = self.pwm1.get_max_duty().into();
            let duty = val as u32 * max / core::i16::MAX as u32;
            self.pwm1.set_duty(Pwm1::Duty::from(duty as u16));
            self.pwm2.set_duty(Pwm2::Duty::from(0));
        } else {
            let max = self.pwm2.get_max_duty().into();
            let duty = (-val) as u32 * max / core::i16::MAX as u32;
            self.pwm1.set_duty(Pwm1::Duty::from(0));
            self.pwm2.set_duty(Pwm2::Duty::from(duty as u16));
        }
    }
}

type SoundCard = PwmSoundCard<pwm::Pwm<stm32::TIM2, pwm::C1>, pwm::Pwm<stm32::TIM2, pwm::C2>>;
static mut SOUND_CARD: Option<SoundCard> = None;
static mut OSCILLATOR: Option<softsynth::Oscillator> = None;

#[entry]
fn main() -> ! {
    let device = stm32::Peripherals::take().unwrap();
    let mut core = cortex_m::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let c2 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
    let c3 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let c4 = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);
    let mut pwm = device.TIM2.pwm(
        (c1, c2, c3, c4),
        &mut afio.mapr,
        (72_000_000 / 256).hz(),
        clocks,
        &mut rcc.apb1,
    );

    pwm.0.enable();
    pwm.1.enable();
    let sound_card = PwmSoundCard {
        pwm1: pwm.0,
        pwm2: pwm.1,
    };
    unsafe {
        SOUND_CARD = Some(sound_card);
    }

    let mut oscillator = Oscillator::default();
    oscillator.set_freq(440);
    unsafe {
        OSCILLATOR = Some(oscillator);
    }

    core.SYST.set_clock_source(SystClkSource::Core);
    core.SYST.set_reload(72_000_000 / RATE);
    core.SYST.enable_interrupt();
    core.SYST.enable_counter();

    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
fn SysTick() {
    let sound_card = unsafe { SOUND_CARD.as_mut().unwrap() };
    let oscillator = unsafe { OSCILLATOR.as_mut().unwrap() };

    sound_card.set(oscillator.get());
    oscillator.advance();
}
