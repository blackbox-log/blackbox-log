mod event;

use alloc::vec::Vec;

pub use self::event::Event;
use super::{
    FrameKind, GpsHomeFrame, Headers, MainFrame, ParseError, ParseResult, Reader, SlowFrame,
};

#[derive(Debug, Clone)]
pub struct Data {
    pub(crate) events: Vec<Event>,
    pub(crate) main_frames: Vec<(MainFrame, usize)>,
    pub(crate) slow_frames: Vec<SlowFrame>,
    // pub(crate) gps_frames: Vec<Frame>,
    pub(crate) gps_home_frames: Vec<GpsHomeFrame>,
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
    pub gps_home: usize,
}

impl Data {
    pub(crate) fn to_stats(&self) -> Stats {
        Stats {
            counts: FrameCounts {
                main: self.main_frames.len(),
                slow: self.slow_frames.len(),
                gps_home: self.gps_home_frames.len(),
            },
        }
    }

    pub fn parse(mut data: Reader, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut main_frames = Vec::new();
        let mut slow_frames = Vec::new();
        // let gps_frames = Vec::new();
        let mut gps_home_frames = Vec::new();

        slow_frames.push(headers.slow_frames.default_frame(headers));

        let mut restore;
        let mut last_kind = None;
        while let Some(byte) = data.read_u8() {
            restore = data.get_restore_point();

            let Some(kind) = FrameKind::from_byte(byte) else {
                tracing::debug!("found invalid frame byte: 0x{byte:0>2x}");

                let last_kind = last_kind.take();
                match last_kind {
                    Some(FrameKind::Event) => {
                        events.pop();
                    }
                    Some(FrameKind::Intra | FrameKind::Inter) => {
                        main_frames.pop();
                    }
                    Some(FrameKind::Slow) => {
                        slow_frames.pop();
                    }
                    Some(FrameKind::GpsHome) => {
                        gps_home_frames.pop();
                    }
                    Some(FrameKind::Gps) | None => {}
                };

                if last_kind.is_some() {
                    data.restore(restore);
                }

                skip_to_frame(&mut data);
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
                FrameKind::Gps => headers.gps_frames.as_ref().map_or_else(
                    || {
                        tracing::error!("found GPS frame without GPS frame definition");
                        Err(ParseError::Corrupted)
                    },
                    |gps| gps.parse(&mut data, headers).map(|_| ()),
                ),
                FrameKind::GpsHome => headers.gps_home_frames.as_ref().map_or_else(
                    || {
                        tracing::error!("found GPS home frame without GPS frame definition");
                        Err(ParseError::Corrupted)
                    },
                    |gps_home| {
                        gps_home
                            .parse(&mut data, headers)
                            .map(|frame| gps_home_frames.push(frame))
                    },
                ),
                FrameKind::Slow => headers
                    .slow_frames
                    .parse(&mut data, headers)
                    .map(|frame| slow_frames.push(frame)),
            };

            match result {
                Ok(()) => {
                    last_kind = Some(kind);
                }
                Err(ParseError::Corrupted) => {
                    tracing::debug!("found corrupted {kind:?} frame");
                    data.restore(restore);
                    skip_to_frame(&mut data);
                }
                Err(ParseError::UnexpectedEof) => {
                    tracing::debug!("found unexpected end of file in data section");
                    break;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(Self {
            events,
            main_frames,
            slow_frames,
            // gps_frames,
            gps_home_frames,
        })
    }
}

#[cold]
fn skip_to_frame(data: &mut Reader) {
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
}
