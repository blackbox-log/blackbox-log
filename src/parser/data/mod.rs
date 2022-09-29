mod event;

pub use event::Event;

use super::{Config, DataFrameKind, FrameKind, Headers, MainFrame, ParseResult, Reader, SlowFrame};
use alloc::vec::Vec;

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Data {
    pub(crate) events: Vec<Event>,
    pub(crate) main_frames: Vec<MainFrame>,
    // pub(crate) gps_frames: Vec<Frame>,
    // pub(crate) gps_home_frames: Vec<Frame>,
    pub(crate) slow_frames: Vec<SlowFrame>,
}

impl Data {
    pub fn parse(data: &mut Reader, config: &Config, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut main_frames = Vec::new();
        // let gps_frames = Vec::new();
        // let gps_home_frames = Vec::new();
        let mut slow_frames = Vec::new();

        // tracing::info!("data parsing starting at 0x{:0>6x}", log.consumed_bytes());
        while let Some(byte) = data.bytes().read_u8() {
            let kind = FrameKind::from_byte(byte).unwrap_or_else(|| {
                // eprintln!();
                // eprintln!("consumed_bytes = 0x{:0>6x}", log.consumed_bytes());

                #[cfg(feature = "std")]
                {
                    use std::iter;

                    let lines = 4;
                    let bytes_per_line = 8;
                    let bytes = iter::once(byte)
                        .chain(data.bytes().iter())
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
                FrameKind::Data(DataFrameKind::Intra) => {
                    let current_idx = main_frames.len();
                    let last = current_idx.checked_sub(1).and_then(|i| main_frames.get(i));
                    let frame = headers
                        .main_frames
                        .parse_intra(data, config, headers, last)?;
                    main_frames.push(frame);
                }
                FrameKind::Data(DataFrameKind::Inter) => {
                    let current_idx = main_frames.len();
                    let last = current_idx.checked_sub(1).and_then(|i| main_frames.get(i));
                    let last_last = current_idx.checked_sub(2).and_then(|i| main_frames.get(i));

                    let skipped = 0; // FIXME

                    let frame = headers
                        .main_frames
                        .parse_inter(data, config, headers, last, last_last, skipped)?;
                    main_frames.push(frame);
                }
                FrameKind::Data(DataFrameKind::Gps) => todo!("handle gps frames"),
                FrameKind::Data(DataFrameKind::GpsHome) => todo!("handle gps home frames"),
                FrameKind::Data(DataFrameKind::Slow) => {
                    let frame = headers.slow_frames.parse(data, config, headers)?;
                    slow_frames.push(frame);
                }
            }
        }

        Ok(Self {
            events,
            main_frames,
            // gps_frames,
            // gps_home_frames,
            slow_frames,
        })
    }
}
