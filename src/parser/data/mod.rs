mod event;
mod frame;

use bitter::BitReader;
pub use event::Event;
pub use frame::{Frame, FrameKind};

use super::Headers;
use crate::{ParseResult, Reader};
use std::iter;

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Data {
    pub(crate) events: Vec<Event>,
    pub(crate) frames: Vec<Frame>,
}

impl Data {
    pub fn parse(data: &mut Reader, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut frames = Vec::new();

        // tracing::info!("data parsing starting at 0x{:0>6x}", log.consumed_bytes());
        while let Some(byte) = data.read_u8() {
            let kind = FrameKind::from_byte(byte).unwrap_or_else(|| {
                // eprintln!();
                // eprintln!("consumed_bytes = 0x{:0>6x}", log.consumed_bytes());

                let lines = 4;
                let bytes_per_line = 8;
                let bytes = iter::once(byte)
                    .chain(iter::from_fn(|| data.read_u8()))
                    .take(lines * bytes_per_line)
                    .collect::<Vec<u8>>();

                for chunk in bytes.chunks_exact(bytes_per_line) {
                    let line = chunk
                        .iter()
                        .map(|x| format!("0x{x:0>2x}"))
                        .collect::<Vec<_>>();
                    let line = line.join(" ");

                    eprintln!("{line}");
                }

                todo!();
            });

            if kind == FrameKind::Event {
                let event = Event::parse(data)?;

                if event == Event::End {
                    tracing::trace!("found the end event");
                    break;
                }

                events.push(event);
            } else {
                let frame_def = match kind {
                    FrameKind::Intra => &headers.frames.intraframe,
                    FrameKind::Inter => &headers.frames.interframe,
                    FrameKind::Slow => &headers.frames.slow,
                    other @ (FrameKind::Gps | FrameKind::GpsHome) => {
                        todo!("unhandled frame type: {other:?}")
                    }
                    FrameKind::Event => unreachable!(),
                };

                let frame = Frame::parse(data, headers, frame_def)?;
                frames.push(frame);
            };
        }

        Ok(Self { events, frames })
    }
}
