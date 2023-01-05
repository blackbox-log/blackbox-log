//! Types for the data section of blackbox logs.

use crate::event::Event;
use crate::frame::gps::{GpsFrame, RawGpsFrame};
use crate::frame::main::{MainFrame, RawMainFrame};
use crate::frame::slow::{RawSlowFrame, SlowFrame};
use crate::frame::{DataFrameKind, FrameKind, GpsHomeFrame};
use crate::parser::InternalError;
use crate::{Headers, Reader};

/// An pseudo-event-based parser for the data section of blackbox logs.
#[derive(Debug)]
pub struct DataParser<'data, 'headers> {
    headers: &'headers Headers<'data>,
    data: Reader<'data>,
    stats: Stats,
    main_frames: MainFrameHistory,
    gps_home_frame: Option<GpsHomeFrame>,
    done: bool,
}

impl<'data, 'headers> DataParser<'data, 'headers> {
    /// Constructs a new parser without beginning parsing.
    pub fn new(data: Reader<'data>, headers: &'headers Headers<'data>) -> Self {
        Self {
            headers,
            data,
            stats: Stats::default(),
            main_frames: MainFrameHistory::default(),
            gps_home_frame: None,
            done: false,
        }
    }

    /// Returns the current stats.
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// Returns `true` if the parser has reached the end of the log.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Continues parsing until the next [`ParseEvent`] can be returned. Returns
    /// `None` if the parser finds the end of the log.
    pub fn next<'parser>(&'parser mut self) -> Option<ParseEvent<'data, 'headers, 'parser>> {
        if self.done {
            return None;
        }

        loop {
            let byte = self.data.read_u8()?;
            let restore = self.data.get_restore_point();

            let Some(kind) = FrameKind::from_byte(byte) else {
                skip_to_frame(&mut self.data);
                continue;
            };

            tracing::trace!("trying to parse {kind:?} frame");

            let result = match kind {
                FrameKind::Event => Event::parse(&mut self.data).map(InternalFrame::Event),
                FrameKind::Data(DataFrameKind::Intra | DataFrameKind::Inter) => {
                    RawMainFrame::parse(&mut self.data, self.headers, kind, &self.main_frames)
                        .map(InternalFrame::Main)
                }
                FrameKind::Data(DataFrameKind::Slow) => self
                    .headers
                    .slow_frame_def
                    .parse(&mut self.data, self.headers)
                    .map(InternalFrame::Slow),
                FrameKind::Data(DataFrameKind::Gps) => {
                    self.headers.gps_frame_def.as_ref().map_or_else(
                        || {
                            tracing::debug!("found GPS frame without GPS frame definition");
                            Err(InternalError::Retry)
                        },
                        |gps| {
                            gps.parse(
                                &mut self.data,
                                self.headers,
                                self.main_frames.last().map(|frame| frame.time),
                                self.gps_home_frame.as_ref(),
                            )
                            .map(InternalFrame::Gps)
                        },
                    )
                }
                FrameKind::Data(DataFrameKind::GpsHome) => {
                    self.headers.gps_home_frame_def.as_ref().map_or_else(
                        || {
                            tracing::debug!(
                                "found GPS home frame without GPS home frame definition"
                            );
                            Err(InternalError::Retry)
                        },
                        |gps_home| {
                            gps_home
                                .parse(&mut self.data, self.headers)
                                .map(InternalFrame::GpsHome)
                        },
                    )
                }
            };

            match result {
                // Check for a good frame kind byte, or EOF
                Ok(frame)
                    if self
                        .data
                        .peek()
                        .map_or(true, |byte| FrameKind::from_byte(byte).is_some()) =>
                {
                    match frame {
                        InternalFrame::Event(event) => {
                            if matches!(event, Event::End { .. }) {
                                self.done = true;
                            }

                            self.stats.counts.event += 1;
                            return Some(ParseEvent::Event(event));
                        }
                        InternalFrame::Main(main) => {
                            self.stats.counts.main += 1;
                            let main = self.main_frames.push(main);

                            return Some(ParseEvent::Main(MainFrame::new(self.headers, main)));
                        }
                        InternalFrame::Slow(slow) => {
                            self.stats.counts.slow += 1;
                            return Some(ParseEvent::Slow(SlowFrame::new(self.headers, slow)));
                        }
                        InternalFrame::Gps(gps) => {
                            self.stats.counts.gps += 1;
                            return Some(ParseEvent::Gps(GpsFrame::new(self.headers, gps)));
                        }
                        InternalFrame::GpsHome(gps_home) => {
                            self.stats.counts.gps_home += 1;
                            self.gps_home_frame = Some(gps_home);
                            continue;
                        }
                    }
                }
                Ok(_) | Err(InternalError::Retry) => {
                    tracing::debug!("found corrupted {kind:?} frame");
                    self.data.restore(restore);
                    skip_to_frame(&mut self.data);
                }
                Err(InternalError::Eof) => {
                    tracing::debug!("found unexpected end of file in data section");
                    return None;
                }
            }
        }
    }
}

/// Statistics about a decoded log.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Stats {
    /// The number of valid frames found of each type.
    pub counts: FrameCounts,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FrameCounts {
    pub event: usize,
    pub main: usize,
    pub slow: usize,
    pub gps: usize,
    pub gps_home: usize,
}

#[derive(Debug)]
pub enum ParseEvent<'data, 'headers, 'parser> {
    Event(Event),
    Main(MainFrame<'data, 'headers, 'parser>),
    Slow(SlowFrame<'data, 'headers>),
    Gps(GpsFrame<'data, 'headers>),
}

#[cold]
fn skip_to_frame(data: &mut Reader) {
    data.skip_until_any(
        &[
            FrameKind::Event,
            FrameKind::Data(DataFrameKind::Intra),
            FrameKind::Data(DataFrameKind::Slow),
            FrameKind::Data(DataFrameKind::Gps),
            FrameKind::Data(DataFrameKind::GpsHome),
        ]
        .map(u8::from),
    );
}

#[derive(Debug, Default)]
pub(crate) struct MainFrameHistory {
    history: [Option<RawMainFrame>; 2],
    index_new: usize,
}

impl MainFrameHistory {
    #[inline(always)]
    fn index_old(&self) -> usize {
        (self.index_new + 1) % self.history.len()
    }

    fn push(&mut self, frame: RawMainFrame) -> &RawMainFrame {
        self.index_new = self.index_old();
        self.history[self.index_new] = Some(frame);
        self.last().unwrap()
    }

    pub(crate) fn last(&self) -> Option<&RawMainFrame> {
        self.history[self.index_new].as_ref()
    }

    pub(crate) fn last_last(&self) -> Option<&RawMainFrame> {
        self.history[self.index_old()].as_ref()
    }
}

#[derive(Debug)]
enum InternalFrame {
    Event(Event),
    Main(RawMainFrame),
    Slow(RawSlowFrame),
    Gps(RawGpsFrame),
    GpsHome(GpsHomeFrame),
}
