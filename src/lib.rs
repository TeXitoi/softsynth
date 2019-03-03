#![no_std]

mod adsr;
mod oscillator;

pub use adsr::Adsr;
pub use oscillator::Oscillator;

pub const RATE: u32 = 48000;
pub const MAX_VOL: i16 = core::i16::MAX;

pub enum Action {
    Vol(i16),
    Start(u16),
    Stop,
}

pub trait Sound {
    fn set_freq(&mut self, freq: u16);
    fn get(&self) -> i16;
    fn advance(&mut self);
    fn step(&mut self) -> i16 {
        let res = self.get();
        self.advance();
        res
    }
    fn stop(&mut self);
    fn set_vol(&mut self, vol: i16);
    fn modify(&mut self, action: &Action) {
        match action {
            Action::Vol(vol) => self.set_vol(*vol),
            Action::Start(freq) => self.set_freq(*freq),
            Action::Stop => self.stop(),
        }
    }
}
