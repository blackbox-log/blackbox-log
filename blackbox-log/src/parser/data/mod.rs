mod event;

use alloc::vec::Vec;

pub use self::event::Event;
use super::{FrameKind, Headers, MainFrame, ParseError, ParseResult, Reader, SlowFrame};

#[derive(Debug, Clone)]
pub struct Data {
    pub(crate) events: Vec<Event>,
    pub(crate) main_frames: Vec<(MainFrame, usize)>,
    // pub(crate) gps_frames: Vec<Frame>,
    // pub(crate) gps_home_frames: Vec<Frame>,
    pub(crate) slow_frames: Vec<SlowFrame>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Stats {
    pub counts: FrameCounts,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FrameCounts {
    pub main: usize,
    pub slow: usize,
}

impl Data {
    pub(crate) fn to_stats(&self) -> Stats {
        Stats {
            counts: FrameCounts {
                main: self.main_frames.len(),
                slow: self.slow_frames.len(),
            },
        }
    }

    pub fn parse(mut data: Reader, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut main_frames = Vec::new();
        // let gps_frames = Vec::new();
        // let gps_home_frames = Vec::new();
        let mut slow_frames = Vec::new();

        slow_frames.push(headers.slow_frames.default_frame(headers));

        let mut last_kind = None;
        while let Some(kind) = data.read_u8().map(FrameKind::from_byte) {
            // TODO (rust 1.65): let-else
            let kind = if let Some(kind) = kind {
                kind
            } else {
                tracing::error!("found invalid frame byte");
                match last_kind.take() {
                    Some(FrameKind::Event) => {
                        events.pop();
                    }
                    Some(FrameKind::Intra | FrameKind::Inter) => {
                        main_frames.pop();
                    }
                    Some(FrameKind::Slow) => {
                        slow_frames.pop();
                    }
                    Some(FrameKind::Gps | FrameKind::GpsHome) | None => {}
                };

                data.skip_until_any(
                    &[
                        FrameKind::Event,
                        FrameKind::Intra,
                        FrameKind::Slow,
                        FrameKind::Gps,
                        FrameKind::GpsHome,
                    ]
                    .map(u8::from),
                );

                continue;
            };

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
                    let frame = MainFrame::parse(&mut data, kind, &main_frames, headers);
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
                Ok(()) => {
                    last_kind = Some(kind);
                }
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
