use crate::parser::{
    Data, Event, Headers, MainFrame, MainUnit, MainValue, ParseResult, Reader, SlowFrame, SlowUnit,
    SlowValue,
};

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
    pub fn parse(data: &'data [u8]) -> ParseResult<Self> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;

        let data = Data::parse(data, &headers)?;

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

    pub fn main_fields(&self) -> impl Iterator<Item = (&str, MainUnit)> {
        self.headers.main_fields()
    }

    pub fn slow_fields(&self) -> impl Iterator<Item = (&str, SlowUnit)> {
        self.headers.slow_fields()
    }
}

#[derive(Debug)]
pub struct FrameIter<'log, 'data> {
    log: &'log Log<'data>,
    index: usize,
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
            headers: self.log.headers(),
            main,
            slow,
        })
    }
}

impl<'log, 'data> core::iter::FusedIterator for FrameIter<'log, 'data> {}

#[derive(Debug)]
pub struct FrameView<'log, 'data> {
    headers: &'log Headers<'data>,
    main: &'log MainFrame,
    slow: &'log SlowFrame,
}

impl<'log, 'data> FrameView<'log, 'data> {
    pub fn iter_main(&self) -> impl Iterator<Item = MainValue> + '_ {
        self.main.iter(self.headers)
    }

    pub fn iter_slow(&self) -> impl Iterator<Item = SlowValue> + '_ {
        self.slow.iter()
    }
}
