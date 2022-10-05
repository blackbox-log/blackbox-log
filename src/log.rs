use core::iter;

use crate::parser::{self, Data, Event, Frame, Headers, MainFrame, ParseResult, Reader, SlowFrame};
use crate::units::{Unit, UnitKind};

#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
}

impl<'data> Log<'data> {
    /// Attempts to parse a single blackbox log
    ///
    /// **Note**: This assumes that `data` is already aligned to the start of
    /// the log and will return an error if it is not.
    pub fn parse<'config>(config: &'config parser::Config, data: &'data [u8]) -> ParseResult<Self> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;

        let data = Data::parse(data, config, &headers)?;

        Ok(Self { headers, data })
    }

    pub fn headers(&self) -> &Headers<'data> {
        &self.headers
    }

    pub fn events(&self) -> &[Event] {
        &self.data.events
    }

    pub fn iter_frames(&self) -> FrameIter {
        FrameIter {
            log: self,
            index: 0,
        }
    }

    pub fn fields(&self) -> impl Iterator<Item = (&str, UnitKind)> {
        self.headers.main_fields().chain(self.headers.slow_fields())
    }
}

#[derive(Debug)]
pub struct FrameIter<'a, 'data> {
    log: &'a Log<'data>,
    index: usize,
}

impl<'log, 'data> Iterator for FrameIter<'log, 'data> {
    type Item = FieldIter<'log, 'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.log.data.main_frames.len() {
            return None;
        }

        let (main, slow) = &self.log.data.main_frames[self.index];
        let slow = &self.log.data.slow_frames[*slow];
        self.index += 1;

        Some(FieldIter {
            headers: &self.log.headers,
            main,
            slow,
            field: 0,
        })
    }
}

#[derive(Debug)]
pub struct FieldIter<'a, 'data> {
    headers: &'a Headers<'data>,
    main: &'a MainFrame,
    slow: &'a SlowFrame,
    field: usize,
}

impl<'a, 'data> Iterator for FieldIter<'a, 'data> {
    type Item = Unit;

    fn next(&mut self) -> Option<Self::Item> {
        let main_len = self.main.len();
        let slow_len = self.slow.len();

        let (unit, value) = match self.field {
            0 => (
                self.headers.main_frames.iteration.unit,
                self.main.iteration().into(),
            ),
            1 => (self.headers.main_frames.time.unit, self.main.time()),
            i if i < main_len => {
                let i = i - 2;
                (
                    self.headers.main_frames.fields[i].unit,
                    self.main.values()[i],
                )
            }
            i if i < (main_len + slow_len) => {
                let i = i - main_len;
                (self.headers.slow_frames.0[i].unit, self.slow.values()[i])
            }
            _ => {
                return None;
            }
        };

        self.field += 1;
        Some(unit.with_value(value, self.headers))
    }
}

impl<'a, 'data> iter::FusedIterator for FieldIter<'a, 'data> {}
