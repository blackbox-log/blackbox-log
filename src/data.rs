//! Types for the data section of blackbox logs.

use crate::event::Event;
use crate::filter::AppliedFilter;
use crate::frame::gps::{GpsFrame, RawGpsFrame};
use crate::frame::main::{MainFrame, RawMainFrame};
use crate::frame::slow::{RawSlowFrame, SlowFrame};
use crate::frame::{self, DataFrameKind, FilteredFrameDef, FrameKind, GpsHomeFrame};
use crate::parser::InternalError;
use crate::{FilterSet, Headers, Reader};

/// An pseudo-event-based parser for the data section of blackbox logs.
#[derive(Debug)]
pub struct DataParser<'data, 'headers> {
    headers: &'headers Headers<'data>,
    main_filter: AppliedFilter,
    slow_filter: AppliedFilter,
    gps_filter: AppliedFilter,
    data: Reader<'data>,
    data_len: usize,
    stats: Stats,
    main_frames: MainFrameHistory,
    gps_home_frame: Option<GpsHomeFrame>,
    done: bool,
}

impl<'data, 'headers> DataParser<'data, 'headers> {
    pub(crate) fn new(
        data: Reader<'data>,
        headers: &'headers Headers<'data>,
        filters: &FilterSet,
    ) -> Self {
        let data_len = data.remaining();

        Self {
            headers,
            main_filter: filters.main.apply(headers.main_frame_def()),
            slow_filter: filters.slow.apply(headers.slow_frame_def()),
            gps_filter: headers
                .gps_frame_def()
                .map(|def| filters.gps.apply(def))
                .unwrap_or_default(),
            data,
            data_len,
            stats: Stats::default(),
            main_frames: MainFrameHistory::default(),
            gps_home_frame: None,
            done: false,
        }
    }

    pub fn main_frame_def<'a>(&'a self) -> FilteredFrameDef<'a, frame::MainFrameDef<'data>> {
        FilteredFrameDef::new(self.headers.main_frame_def(), &self.main_filter)
    }

    pub fn slow_frame_def<'a>(&'a self) -> FilteredFrameDef<'a, frame::SlowFrameDef<'data>> {
        FilteredFrameDef::new(self.headers.slow_frame_def(), &self.slow_filter)
    }

    pub fn gps_frame_def<'a>(&'a self) -> Option<FilteredFrameDef<'a, frame::GpsFrameDef<'data>>> {
        self.headers
            .gps_frame_def()
            .map(|def| FilteredFrameDef::new(def, &self.gps_filter))
    }

    /// Returns the current stats.
    #[inline]
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// Returns `true` if the parser has reached the end of the log.
    #[inline]
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Continues parsing until the next [`ParserEvent`] can be returned.
    /// Returns `None` if the parser finds the end of the log.
    pub fn next<'parser>(&'parser mut self) -> Option<ParserEvent<'data, 'headers, 'parser>> {
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
                    .slow_frame_def()
                    .parse(&mut self.data, self.headers)
                    .map(InternalFrame::Slow),
                FrameKind::Data(DataFrameKind::Gps) => {
                    self.headers.gps_frame_def().as_ref().map_or_else(
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
                    self.headers.gps_home_frame_def().as_ref().map_or_else(
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

            self.stats.progress = 1. - ((self.data.remaining() as f32) / (self.data_len as f32));

            match result {
                // Check for a good frame kind byte, or EOF
                Ok(frame)
                    if self
                        .data
                        .peek()
                        .is_none_or(|byte| FrameKind::from_byte(byte).is_some()) =>
                {
                    match frame {
                        InternalFrame::Event(event) => {
                            if matches!(event, Event::End { .. }) {
                                self.done = true;
                                self.stats.progress = 1.;
                            }

                            self.stats.counts.event += 1;
                            return Some(ParserEvent::Event(event));
                        }
                        InternalFrame::Main(main) => {
                            self.stats.counts.main += 1;
                            let main = self.main_frames.push(main);

                            return Some(ParserEvent::Main(MainFrame::new(
                                self.headers,
                                main,
                                &self.main_filter,
                            )));
                        }
                        InternalFrame::Slow(slow) => {
                            self.stats.counts.slow += 1;
                            return Some(ParserEvent::Slow(SlowFrame::new(
                                self.headers,
                                slow,
                                &self.slow_filter,
                            )));
                        }
                        InternalFrame::Gps(gps) => {
                            self.stats.counts.gps += 1;
                            return Some(ParserEvent::Gps(GpsFrame::new(
                                self.headers,
                                gps,
                                &self.gps_filter,
                            )));
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
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Stats {
    /// The number of valid frames found of each type.
    pub counts: FrameCounts,

    /// The approximate percentage of the log data parsed so far as a number in
    /// the range `0..=1`.
    ///
    /// If there is extra data between logs this could massively underestimate,
    /// but it will not overestimate.
    pub progress: f32,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
pub struct FrameCounts {
    pub event: usize,
    pub main: usize,
    pub slow: usize,
    pub gps: usize,
    pub gps_home: usize,
}

/// An item parsed from the data section of a blackbox log.
///
/// See [`DataParser::next`].
#[derive(Debug)]
pub enum ParserEvent<'data, 'headers, 'parser> {
    Event(Event),
    Main(MainFrame<'data, 'headers, 'parser>),
    Slow(SlowFrame<'data, 'headers, 'parser>),
    Gps(GpsFrame<'data, 'headers, 'parser>),
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
