mod event;

use alloc::vec::Vec;

pub use self::event::Event;
use super::{FrameKind, Headers, MainFrame, ParseError, ParseResult, Reader, SlowFrame};

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Data {
    pub(crate) events: Vec<Event>,
    pub(crate) main_frames: Vec<(MainFrame, usize)>,
    // pub(crate) gps_frames: Vec<Frame>,
    // pub(crate) gps_home_frames: Vec<Frame>,
    pub(crate) slow_frames: Vec<SlowFrame>,
}

impl Data {
    pub fn parse(mut data: Reader, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut main_frames = Vec::new();
        // let gps_frames = Vec::new();
        // let gps_home_frames = Vec::new();
        let mut slow_frames = Vec::new();

        slow_frames.push(headers.slow_frames.default_frame(headers));

        while let Some(byte) = data.read_u8() {
            let kind = FrameKind::from_byte(byte).unwrap_or_else(|| {
                #[cfg(feature = "std")]
                {
                    use core::iter;

                    let lines = 4;
                    let bytes_per_line = 8;
                    let bytes = iter::once(byte)
                        .chain(data.iter())
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

            let result = match kind {
                FrameKind::Event => match Event::parse_into(&mut data, &mut events) {
                    Ok(event::EventKind::End) => {
                        tracing::trace!("found the end event");
                        break;
                    }
                    Ok(_) => Ok(()),
                    Err(err) => Err(err),
                },
                FrameKind::Intra | FrameKind::Inter => {
                    let get_main_frame = |i| main_frames.get(i).map(|(frame, _)| frame);

                    let current_idx = main_frames.len();
                    let last = current_idx.checked_sub(1).and_then(get_main_frame);
                    let main = &headers.main_frames;

                    let frame = if kind == FrameKind::Intra {
                        main.parse_intra(&mut data, headers, last)
                    } else {
                        let last_last = current_idx.checked_sub(2).and_then(get_main_frame);
                        let skipped = 0; // FIXME

                        main.parse_inter(&mut data, headers, last, last_last, skipped)
                    };

                    frame.map(|frame| main_frames.push((frame, slow_frames.len() - 1)))
                }
                FrameKind::Gps => {
                    if let Some(ref gps) = headers.gps_frames {
                        gps.parse(&mut data, headers).map(|_| ())
                    } else {
                        tracing::error!("found GPS frame without GPS frame definition");
                        return Err(ParseError::Corrupted);
                    }
                }
                FrameKind::GpsHome => {
                    if let Some(ref gps_home) = headers.gps_home_frames {
                        gps_home.parse(&mut data, headers).map(|_| ())
                    } else {
                        tracing::error!("found GPS home frame without GPS frame definition");
                        return Err(ParseError::Corrupted);
                    }
                }
                FrameKind::Slow => headers
                    .slow_frames
                    .parse(&mut data, headers)
                    .map(|frame| slow_frames.push(frame)),
            };

            match result {
                Ok(()) => {}
                Err(ParseError::UnexpectedEof) => {
                    tracing::warn!("found unexpected end of file");
                    break;
                }
                Err(err) => return Err(err),
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
