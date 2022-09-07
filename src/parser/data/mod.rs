mod event;
mod frame;

pub use event::Event;
pub use frame::Frame;

use super::{DataFrameKind, FrameKind, Headers, ParseResult};
use crate::Reader;
use bitter::BitReader;
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

            match kind {
                FrameKind::Event => {
                    let event = Event::parse(data)?;
                    let is_end = event == Event::End;
                    events.push(event);

                    if is_end {
                        tracing::trace!("found the end event");
                        break;
                    }
                }
                FrameKind::Data(data_kind) => {
                    let frame_def = match data_kind {
                        DataFrameKind::Intra => headers.frames.intra(),
                        DataFrameKind::Inter => headers.frames.inter(),
                        DataFrameKind::Slow => headers.frames.slow(),
                        other @ (DataFrameKind::Gps | DataFrameKind::GpsHome) => {
                            todo!("unhandled frame type: {other:?}")
                        }
                    };

                    let current = frames.len();
                    let last = current.checked_sub(1).and_then(|i| frames.get(i));
                    let last_last = current.checked_sub(2).and_then(|i| frames.get(i));
                    let frame = Frame::parse(data, headers, frame_def, last, last_last)?;
                    frames.push(frame);
                }
            }
        }

        Ok(Self { events, frames })
    }
}
