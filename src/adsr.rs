use crate::{Sound, MAX_VOL, RATE};

pub struct Adsr<S> {
    sound: S,
    attack_ms: u32,
    decay_ms: u32,
    sustain: i16,
    release_ms: u32,
    vol: i16,
    state: AdsrState,
}
#[derive(Copy, Clone)]
enum AdsrState {
    Stop,
    Attack(u32),
    Decay(u32),
    Sustain,
    Release(u32),
}

fn as_ticks(ms: u32) -> u32 {
    ms * RATE / 1000
}
impl<S: Sound> Adsr<S> {
    pub fn new(mut sound: S, attack_ms: u32, decay_ms: u32, sustain: i16, release_ms: u32) -> Self {
        sound.set_vol(0);
        sound.stop();
        Self {
            sound,
            attack_ms,
            decay_ms,
            sustain,
            release_ms,
            vol: MAX_VOL,
            state: AdsrState::Stop,
        }
    }
    fn sustain_vol(&self) -> i16 {
        (self.vol as i32 * self.sustain as i32 / MAX_VOL as i32) as i16
    }
}

impl<S: Sound> Sound for Adsr<S> {
    fn set_freq(&mut self, freq: u16) {
        self.sound.set_freq(freq);
        self.state = AdsrState::Attack(as_ticks(self.attack_ms))
    }
    fn get(&self) -> i16 {
        self.sound.get()
    }
    fn advance(&mut self) {
        use AdsrState::*;
        match self.state {
            Stop => {}
            Attack(ticks) => {
                let total_ticks = as_ticks(self.attack_ms) as i32;
                let vol = self.vol as i32 * (total_ticks - ticks as i32) / total_ticks;
                self.sound.set_vol(vol as i16);
                self.state = match ticks {
                    0 => Decay(as_ticks(self.decay_ms)),
                    ticks => Attack(ticks - 1),
                }
            }
            Decay(ticks) => {
                let total_ticks = as_ticks(self.decay_ms) as i32;
                let sustain_vol = self.sustain_vol() as i32;
                let vol =
                    (self.vol as i32 - sustain_vol) * ticks as i32 / total_ticks + sustain_vol;
                self.sound.set_vol(vol as i16);
                self.state = match ticks {
                    0 => {
                        self.sound.set_vol(sustain_vol as i16);
                        Sustain
                    }
                    ticks => Decay(ticks - 1),
                }
            }
            Sustain => {}
            Release(ticks) => {
                let total_ticks = as_ticks(self.release_ms) as i32;
                let sustain_vol = self.sustain_vol() as i32;
                let vol = sustain_vol * ticks as i32 / total_ticks;
                self.sound.set_vol(vol as i16);
                self.state = match ticks {
                    0 => {
                        self.sound.set_vol(0);
                        self.sound.stop();
                        Stop
                    }
                    ticks => Release(ticks - 1),
                }
            }
        }
        self.sound.advance();
    }
    fn stop(&mut self) {
        self.state = AdsrState::Release(as_ticks(self.release_ms))
    }
    fn set_vol(&mut self, vol: i16) {
        self.vol = vol;
    }
}
