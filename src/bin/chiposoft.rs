use byteorder::{WriteBytesExt, LE};
use chiposoft::{Action, Oscilator, RATE};
use std::io::Write;

struct Event {
    ms: u32,
    chan: usize,
    action: Action,
}
static EVENTS: [Event; 12] = [
    Event {
        ms: 0,
        chan: 0,
        action: Action::Vol(255 / 4),
    },
    Event {
        ms: 0,
        chan: 1,
        action: Action::Vol(255 / 4),
    },
    Event {
        ms: 0,
        chan: 2,
        action: Action::Vol(255 / 4),
    },
    Event {
        ms: 0,
        chan: 3,
        action: Action::Vol(255 / 4),
    },
    Event {
        ms: 0,
        chan: 0,
        action: Action::Start(523),
    },
    Event {
        ms: 2_000,
        chan: 1,
        action: Action::Start(659),
    },
    Event {
        ms: 4_000,
        chan: 2,
        action: Action::Start(784),
    },
    Event {
        ms: 6_000,
        chan: 3,
        action: Action::Start(932),
    },
    Event {
        ms: 10_000,
        chan: 0,
        action: Action::Stop,
    },
    Event {
        ms: 10_000,
        chan: 1,
        action: Action::Stop,
    },
    Event {
        ms: 10_000,
        chan: 2,
        action: Action::Stop,
    },
    Event {
        ms: 10_000,
        chan: 3,
        action: Action::Stop,
    },
];

fn make(song: &[Event]) -> impl core::iter::ExactSizeIterator<Item = i16> + '_ {
    let mut oscilators = [
        Oscilator::default(),
        Oscilator::default(),
        Oscilator::default(),
        Oscilator::default(),
    ];
    let duration = song.iter().map(|e| e.ms).max().unwrap_or(0);
    let mut events = song.iter();
    let mut event = events.next();
    let mut ms = 0;
    (0..RATE * duration / 1000).map(move |t| {
        if t % (RATE / 1000) == 0 {
            loop {
                match event {
                    None => break,
                    Some(e) if e.ms != ms => break,
                    Some(e) => oscilators[e.chan].modify(&e.action),
                }
                event = events.next();
            }
            ms += 1;
        }
        let mut res = oscilators.iter_mut().map(|o| o.step() as i32).sum();
        if res > core::i16::MAX as i32 {
            res = core::i16::MAX as i32;
        } else if res < core::i16::MIN as i32 {
            res = core::i16::MIN as i32;
        }
        res as i16
    })
}

fn main() -> std::io::Result<()> {
    let v = make(&EVENTS);
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
