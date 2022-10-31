use alloc::vec::Vec;
use core::iter;

use crate::parser::{
    to_base_field, Data, Event, Headers, MainFrame, MainUnit, MainValue, ParseResult, Reader,
    SlowFrame, SlowUnit, SlowValue,
};

#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
    filter: Filter,
}

#[derive(Debug)]
struct Filter {
    main: Vec<bool>,
    slow: Vec<bool>,
}

impl Filter {
    fn new<S: AsRef<str>>(fields: &[S], headers: &Headers) -> Self {
        let mut fields = fields
            .iter()
            .map(|s| to_base_field(s.as_ref()))
            .collect::<Vec<_>>();
        fields.sort_unstable();

        Self {
            main: headers
                .main_fields()
                .map(|(field, _)| fields.binary_search(&to_base_field(field)).is_ok())
                .collect(),
            slow: headers
                .slow_fields()
                .map(|(field, _)| fields.binary_search(&to_base_field(field)).is_ok())
                .collect(),
        }
    }

    fn new_unfiltered(headers: &Headers) -> Self {
        Self {
            main: iter::repeat(true).take(headers.main_frames.len()).collect(),
            slow: iter::repeat(true).take(headers.slow_frames.len()).collect(),
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

    pub const fn iter_frames(&self) -> FrameIter {
        FrameIter::new(self)
    }

    pub fn main_fields<'a: 'data>(&'a self) -> impl Iterator<Item = (&'data str, MainUnit)> + 'a {
        self.headers
            .main_fields()
            .enumerate()
            .filter_map(|(i, field)| self.filter.main[i].then_some(field))
    }

    pub fn slow_fields<'a: 'data>(&'a self) -> impl Iterator<Item = (&'data str, SlowUnit)> + 'a {
        self.headers
            .slow_fields()
            .enumerate()
            .filter_map(|(i, field)| self.filter.slow[i].then_some(field))
    }
}

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
    type Item = FrameView<'log, 'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.log.data.main_frames.len() {
            return None;
        }

        let (main, slow) = &self.log.data.main_frames[self.index];
        let slow = &self.log.data.slow_frames[*slow];
        self.index += 1;

        Some(FrameView {
            log: self.log,
            main,
            slow,
        })
    }
}

impl<'log, 'data> core::iter::FusedIterator for FrameIter<'log, 'data> {}

#[derive(Debug)]
pub struct FrameView<'log, 'data> {
    log: &'log Log<'data>,
    main: &'log MainFrame,
    slow: &'log SlowFrame,
}

impl<'log, 'data> FrameView<'log, 'data> {
    pub fn iter_main(&self) -> impl Iterator<Item = MainValue> + '_ {
        self.main
            .iter(self.log.headers())
            .enumerate()
            .filter_map(|(i, value)| self.log.filter.main[i].then_some(value))
    }

    pub fn iter_slow(&self) -> impl Iterator<Item = SlowValue> + '_ {
        self.slow
            .iter()
            .enumerate()
            .filter_map(|(i, value)| self.log.filter.slow[i].then_some(value))
    }
}
