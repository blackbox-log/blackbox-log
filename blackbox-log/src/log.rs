use alloc::vec::Vec;
use core::iter::FusedIterator;

use crate::parser::{
    to_base_field, Data, Event, FrameSync, GpsFrame, Headers, MainFrame, ParseResult, Reader,
    SlowFrame, Stats, Unit, Value,
};

#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
    filter: Filter,
}

#[derive(Debug)]
struct Filter {
    main: Vec<usize>,
    slow: Vec<usize>,
    gps: Vec<usize>,
}

impl Filter {
    fn new<S: AsRef<str>>(fields: &[S], headers: &Headers) -> Self {
        let mut fields = fields
            .iter()
            .map(|s| to_base_field(s.as_ref()))
            .collect::<Vec<_>>();
        fields.sort_unstable();

        let filter = |(i, field)| {
            fields
                .binary_search(&to_base_field(field))
                .is_ok()
                .then_some(i)
        };

        Self {
            main: headers
                .main_fields()
                .map(|(name, _)| name)
                .enumerate()
                .filter_map(filter)
                .collect(),
            slow: headers
                .slow_fields()
                .map(|(name, _)| name)
                .enumerate()
                .filter_map(filter)
                .collect(),
            gps: headers
                .gps_fields()
                .map(|(name, _)| name)
                .enumerate()
                .filter_map(filter)
                .collect(),
        }
    }

    #[allow(clippy::redundant_closure_for_method_calls)]
    fn new_unfiltered(headers: &Headers) -> Self {
        Self {
            main: (0..headers.main_frames.len()).collect(),
            slow: (0..headers.slow_frames.len()).collect(),
            gps: (0..headers.gps_frames.as_ref().map_or(0, |def| def.len())).collect(),
        }
    }
}

impl<'data> Log<'data> {
    /// Attempts to parse a single blackbox log
    ///
    /// **Note**: This assumes that `data` is already aligned to the start of
    /// the log and will return an error if it is not.
    pub fn parse(data: &'data [u8]) -> ParseResult<Self> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;

        let data = Data::parse(data, &headers)?;
        let filter = Filter::new_unfiltered(&headers);

        Ok(Self {
            headers,
            data,
            filter,
        })
    }

    pub fn set_filter<S: AsRef<str>>(&mut self, filter: &[S]) {
        self.filter = Filter::new(filter, self.headers());
    }

    pub fn unset_filter(&mut self) {
        self.filter = Filter::new_unfiltered(self.headers());
    }

    pub const fn headers(&self) -> &Headers<'data> {
        &self.headers
    }

    pub fn events(&self) -> &[Event] {
        &self.data.events
    }

    pub fn stats(&self) -> Stats {
        self.data.to_stats()
    }

    pub const fn iter_frames(&self) -> FrameIter {
        FrameIter::new(self)
    }

    pub const fn iter_fields(&self) -> FieldIter {
        FieldIter::new(self)
    }
}

#[derive(Debug)]
pub struct FieldIter<'log, 'data> {
    log: &'log Log<'data>,
    index: usize,
}

impl<'log, 'data> FieldIter<'log, 'data> {
    const fn new(log: &'log Log<'data>) -> Self {
        Self { log, index: 0 }
    }
}

impl<'log: 'data, 'data> Iterator for FieldIter<'log, 'data> {
    type Item = (&'data str, Unit);

    fn next(&mut self) -> Option<Self::Item> {
        let Log {
            headers, filter, ..
        } = self.log;

        let next = get_next(
            self.index,
            filter,
            |index| {
                let (name, unit) = headers.main_frames.get(index).unwrap();
                (name, unit.into())
            },
            |index| {
                let (name, unit) = headers.slow_frames.get(index).unwrap();
                (name, unit.into())
            },
            |index| {
                let def = headers.gps_frames.as_ref().unwrap();
                let (name, unit) = def.get(index).unwrap();
                (name, unit.into())
            },
        )?;

        self.index += 1;
        Some(next)
    }
}

impl<'log: 'data, 'data> FusedIterator for FieldIter<'log, 'data> {}

#[derive(Debug)]
pub struct FrameIter<'log, 'data> {
    log: &'log Log<'data>,
    index: usize,
}

impl<'log, 'data> FrameIter<'log, 'data> {
    const fn new(log: &'log Log<'data>) -> Self {
        Self { log, index: 0 }
    }
}

impl<'log, 'data> Iterator for FrameIter<'log, 'data> {
    type Item = FieldValueIter<'log, 'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.log.data.main_frames.len() {
            return None;
        }

        let FrameSync { main, slow, gps } = &self.log.data.main_frames[self.index];
        let slow = &self.log.data.slow_frames[*slow];
        let gps = gps.map(|index| &self.log.data.gps_frames[index]);
        self.index += 1;

        Some(FieldValueIter {
            log: self.log,
            main,
            slow,
            gps,
            index: 0,
        })
    }
}

impl<'log, 'data> FusedIterator for FrameIter<'log, 'data> {}

#[derive(Debug)]
pub struct FieldValueIter<'log, 'data> {
    log: &'log Log<'data>,
    main: &'log MainFrame,
    slow: &'log SlowFrame,
    gps: Option<&'log GpsFrame>,
    index: usize,
}

impl<'log, 'data> Iterator for FieldValueIter<'log, 'data> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let Log {
            headers, filter, ..
        } = self.log;

        let next = get_next(
            self.index,
            filter,
            |index| self.main.get(index, headers).unwrap().into(),
            |index| self.slow.values[index].into(),
            |index| self.gps.unwrap().get(index).unwrap().into(),
        )?;

        self.index += 1;
        Some(next)
    }
}

impl<'log, 'data> FusedIterator for FieldValueIter<'log, 'data> {}

#[inline]
fn get_next<T>(
    index: usize,
    filter: &Filter,
    get_main: impl Fn(usize) -> T,
    get_slow: impl Fn(usize) -> T,
    get_gps: impl Fn(usize) -> T,
) -> Option<T> {
    let slow = filter.main.len();
    let gps = slow + filter.slow.len();
    let done = gps + filter.gps.len();

    let next = if index < slow {
        let index = filter.main[index];
        get_main(index)
    } else if index < gps {
        let index = filter.slow[index - slow];
        get_slow(index)
    } else if index < done {
        let index = filter.gps[index - gps];
        get_gps(index)
    } else {
        return None;
    };

    Some(next)
}
