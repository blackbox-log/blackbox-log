use alloc::vec::Vec;

use crate::event::{self, Event};
use crate::frame::{FrameKind, GpsFrame, GpsHomeFrame, MainFrame, SlowFrame};
use crate::parser::InternalError;
use crate::{Headers, ParseResult, Reader};

#[derive(Debug, Clone)]
pub struct Data {
    pub(crate) events: Vec<Event>,
    pub(crate) main_frames: Vec<FrameSync>,
    pub(crate) slow_frames: Vec<SlowFrame>,
    pub(crate) gps_frames: Vec<GpsFrame>,
    pub(crate) gps_home_frames: Vec<GpsHomeFrame>,
}

#[derive(Debug, Clone)]
pub(crate) struct FrameSync {
    pub main: MainFrame,
    pub slow: usize,
    pub gps: Option<usize>,
}

impl FrameSync {
    pub(crate) fn new(main: MainFrame, slow: &[SlowFrame], gps: &[GpsFrame]) -> Self {
        Self {
            main,
            slow: slow.len() - 1,
            gps: gps.len().checked_sub(1),
        }
    }
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
    pub gps: usize,
    pub gps_home: usize,
}

impl Data {
    pub(crate) fn to_stats(&self) -> Stats {
        Stats {
            counts: FrameCounts {
                main: self.main_frames.len(),
                slow: self.slow_frames.len(),
                gps: self.gps_frames.len(),
                gps_home: self.gps_home_frames.len(),
            },
        }
    }

    pub fn parse(mut data: Reader, headers: &Headers) -> ParseResult<Self> {
        let mut events = Vec::new();
        let mut main_frames = Vec::new();
        let mut slow_frames = Vec::new();
        let mut gps_frames = Vec::new();
        let mut gps_home_frames = Vec::new();

        slow_frames.push(headers.slow_frames.empty_frame());

        if let Some(def) = &headers.gps_frames {
            gps_frames.push(def.empty_frame());
        }

        let mut restore;
        let mut last_kind = None;
        while let Some(byte) = data.read_u8() {
            restore = data.get_restore_point();

            let Some(kind) = FrameKind::from_byte(byte) else {
                tracing::debug!("found invalid frame byte: {byte:0>#2x}");

                if let Some(last_kind) = last_kind.take() {
                    data.restore(restore);

                    match last_kind {
                        FrameKind::Event => {
                            events.pop();
                        }
                        FrameKind::Intra | FrameKind::Inter => {
                            main_frames.pop();
                        }
                        FrameKind::Slow => {
                            slow_frames.pop();
                        }
                        FrameKind::Gps => {
                            gps_frames.pop();
                        }
                        FrameKind::GpsHome => {
                            gps_home_frames.pop();
                        }
                    };
                }

                skip_to_frame(&mut data);
                continue;
            };

            tracing::trace!("trying to parse {kind:?} frame");

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
                    frame.map(|frame| {
                        main_frames.push(FrameSync::new(frame, &slow_frames, &gps_frames));
                    })
                }
                FrameKind::Gps => headers.gps_frames.as_ref().map_or_else(
                    || {
                        tracing::debug!("found GPS frame without GPS frame definition");
                        Err(InternalError::Retry)
                    },
                    |gps| {
                        gps.parse(
                            &mut data,
                            headers,
                            main_frames.last().map(|sync| &sync.main),
                            gps_home_frames.last(),
                        )
                        .map(|frame| gps_frames.push(frame))
                    },
                ),
                FrameKind::GpsHome => headers.gps_home_frames.as_ref().map_or_else(
                    || {
                        tracing::debug!("found GPS home frame without GPS home frame definition");
                        Err(InternalError::Retry)
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
                Err(InternalError::Retry) => {
                    tracing::debug!("found corrupted {kind:?} frame");
                    data.restore(restore);
                    skip_to_frame(&mut data);
                }
                Err(InternalError::Eof) => {
                    tracing::debug!("found unexpected end of file in data section");
                    break;
                }
                Err(InternalError::Fatal(err)) => return Err(err),
            }
        }

        Ok(Self {
            events,
            main_frames,
            slow_frames,
            gps_frames,
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