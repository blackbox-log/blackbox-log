mod event;
mod frame;

pub use event::Event;
pub use frame::Frame;

use super::Headers;
use crate::ParseResult;
use biterator::Biterator;
use std::io::Read;
use std::iter;
use std::iter::Peekable;

fn print_byte(prefix: &str, byte: u8) {
    eprintln!(
        "{prefix}: 0x{byte:0>2X} ({})",
        char::from_u32(byte as u32).unwrap()
    );
}

#[derive(Debug, Clone)]
pub struct LogData {
    pub(crate) events: Vec<Event>,
    pub(crate) frames: Vec<Frame>,
}

impl LogData {
    pub fn parse<R: Read>(log: &mut Biterator<R>, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut frames = Vec::new();

        eprintln!("consumed_bytes = 0x{:0>6x}", log.consumed_bytes());
        while let Some(byte) = log.bytes().next() {
            match byte {
                b'H' => todo!("header found after frame"),

                b'E' => {
                    let event = Event::parse(log)?;

                    if event == Event::End {
                        eprintln!("found the end");
                        break;
                    }

                    events.push(event);
                }

                b'I' | b'S' => {
                    let frame_def = match byte {
                        b'I' => &headers.frames.intraframe,
                        b'S' => &headers.frames.slow,
                        _ => unreachable!(),
                    };

                    let frame = Frame::parse(log, headers, frame_def)?;
                    frames.push(frame);
                }

                byte => {
                    dbg!(frames);

                    eprintln!();
                    eprintln!("consumed_bytes = 0x{:0>6x}", log.consumed_bytes());

                    let lines = 4;
                    let bytes_per_line = 8;
                    let bytes = iter::once(byte)
                        .chain(log.bytes())
                        .take(lines * bytes_per_line)
                        .collect::<Vec<_>>();

                    for chunk in bytes.chunks_exact(bytes_per_line) {
                        let line = chunk
                            .iter()
                            .map(|x| format!("0x{x:0>2x}"))
                            .collect::<Vec<_>>();
                        let line = line.join(" ");

                        eprintln!("{line}");
                    }

                    todo!();
                }
            }
        }

        Ok(Self { events, frames })
    }
}
