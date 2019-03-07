use byteorder::{WriteBytesExt, LE};
use softsynth::{Adsr, Oscillator, Sound, MAX_VOL, RATE};
use std::io::Write;

fn make(score: &softsynth::songs::Score) -> impl core::iter::ExactSizeIterator<Item = i16> + '_ {
    let mut oscillator = Adsr::new(Oscillator::default(), 10, 20, MAX_VOL / 3 * 2, 5);
    let duration = score.ms_duration();
    let mut events = score.events();
    let mut event = events.next();
    let mut ms = 0;
    let mut next_ms = 0;
    (0..RATE * duration / 1000).map(move |t| {
        if t % (RATE / 1000) == 0 {
            loop {
                match event {
                    None => break,
                    Some(_) if next_ms != 0 => break,
                    Some(e) => {
                        oscillator.modify(&e.into_action());
                        next_ms = e.ms_duration();
                    }
                }
                event = events.next();
            }
            ms += 1;
            next_ms -= 1;
        }
        oscillator.step()
    })
}

fn main() -> std::io::Result<()> {
    let v = make(&softsynth::songs::FRERE_JACQUES);
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    stdout.write_all(b"RIFF")?;
    stdout.write_u32::<LE>(16 + 8 + 8 + 4 + v.len() as u32 * 2)?;
    stdout.write_all(b"WAVE")?;

    stdout.write_all(b"fmt ")?;
    stdout.write_u32::<LE>(16)?;
    stdout.write_u16::<LE>(1)?; // PCM
    stdout.write_u16::<LE>(1)?; // mono
    stdout.write_u32::<LE>(RATE)?; // freq
    stdout.write_u32::<LE>(RATE * 1 * 16 / 8)?; // bytes/s
    stdout.write_u16::<LE>(1 * 16 / 8)?; // bytes/block
    stdout.write_u16::<LE>(16)?; // bits/sample

    stdout.write_all(b"data")?;
    stdout.write_u32::<LE>(v.len() as u32 * 2)?;
    for s in v {
        stdout.write_i16::<LE>(s)?;
    }

    Ok(())
}
