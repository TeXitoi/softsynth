#![no_std]

mod adsr;
mod oscillator;
pub mod pitch;
pub mod songs;

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
    fn vol(&self) -> i16;
    fn get(&self) -> i16;
    fn advance(&mut self);
    fn step(&mut self) -> i16 {
        let res = self.get();
        self.advance();
        res
    }

    fn set_freq(&mut self, freq: u16);
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

pub(crate) fn compute_ratio(from: i16, to: i16, num: u32, denom: u32) -> i16 {
    if denom == 0 {
        return from;
    }
    // Now, denom can't be 0

    // loose precision to not overflow
    let loose = 16u32.saturating_sub(denom.leading_zeros());
    let num = (num >> loose) as i32;
    let denom = (denom >> loose) as i32;

    ((to as i32 - from as i32) * num / denom + from as i32) as i16
}
