use byteorder::{WriteBytesExt, LE};
use chiposoft::{Oscilator, RATE};
use std::io::Write;

fn make() -> impl core::iter::ExactSizeIterator<Item = i16> {
    let mut oscilator = Oscilator::new(440);
    (0..RATE * 10).map(move |t| {
        let res = oscilator.step();
        if t % 480 == 0 {
            if t % RATE < RATE / 2 {
                oscilator.set_freq(oscilator.freq() + 1);
            } else {
                oscilator.set_freq(oscilator.freq() - 1);
            };
        }
        if t % ((RATE * 10) / 256) == 0 && oscilator.vol != 0 {
            oscilator.vol -= 1;
        }
        res
    })
}

fn main() -> std::io::Result<()> {
    let v = make();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    stdout.write_all(b"RIFF")?;
    stdout.write_u32::<LE>(16 + 8 + 8 + 4 + v.len() as u32 * 2)?;
    stdout.write_all(b"WAVE")?;

    stdout.write_all(b"fmt ")?;
    stdout.write_u32::<LE>(16)?;
    stdout.write_u16::<LE>(1)?; // PCM
    stdout.write_u16::<LE>(1)?; // mono
    stdout.write_u32::<LE>(48000)?; // freq
    stdout.write_u32::<LE>(48000 * 1 * 16 / 8)?; // bytes/s
    stdout.write_u16::<LE>(1 * 16 / 8)?; // bytes/block
    stdout.write_u16::<LE>(16)?; // bits/sample

    stdout.write_all(b"data")?;
    stdout.write_u32::<LE>(v.len() as u32 * 2)?;
    for s in v {
        stdout.write_i16::<LE>(s)?;
    }

    Ok(())
}
