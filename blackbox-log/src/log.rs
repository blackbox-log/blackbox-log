use alloc::vec::Vec;
use core::cmp::Ordering;
use core::iter::FusedIterator;

use crate::parser::{
    to_base_field, Data, Event, FrameSync, Headers, ParseResult, Reader, Stats, Unit, Value,
};

#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
}

#[derive(Debug, Clone)]
struct Filter {
    main: Vec<usize>,
    slow: Vec<usize>,
    gps: Vec<usize>,
}

impl Filter {
    fn len(&self) -> usize {
        self.main.len() + self.slow.len() + self.gps.len()
    }

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

    fn merge(&mut self, other: &Self) {
        self.main = intersection(&self.main, &other.main);
        self.slow = intersection(&self.slow, &other.slow);
        self.gps = intersection(&self.gps, &other.gps);
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

        Ok(Self { headers, data })
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

    pub fn data<'log>(&'log self) -> MainView<'log, 'data> {
        let mut filter = Filter::new_unfiltered(&self.headers);

        // Filter out the GPS time field
        if self.headers.gps_frames.is_some() {
            filter.gps.remove(0);
        }

        MainView { log: self, filter }
    }

    pub fn data_with_filter<'log, S: AsRef<str>>(
        &'log self,
        filter: &[S],
    ) -> MainView<'log, 'data> {
        let mut view = self.data();
        view.filter.merge(&Filter::new(filter, &self.headers));
        view
    }
}

#[derive(Debug, Clone)]
pub struct MainView<'log: 'data, 'data> {
    log: &'log Log<'data>,
    filter: Filter,
}

impl<'log: 'data, 'data> MainView<'log, 'data> {
    #[inline]
    fn field_count(&self) -> usize {
        self.filter.len()
    }

    fn frame_count(&self) -> usize {
        self.log.data.main_frames.len()
    }

    pub fn fields(&self) -> FieldIter<'_, Self> {
        FieldIter::new(self)
    }

    pub fn values(&self) -> FrameIter<'_, Self> {
        FrameIter::new(self, self.frame_count())
    }
}

#[derive(Debug)]
pub struct FieldIter<'a, V> {
    view: &'a V,
    index: usize,
}

impl<'a, V> FieldIter<'a, V> {
    const fn new(view: &'a V) -> Self {
        Self { view, index: 0 }
    }
}

impl<'view: 'log, 'log: 'data, 'data> Iterator for FieldIter<'view, MainView<'log, 'data>> {
    type Item = (&'data str, Unit);

    fn next(&mut self) -> Option<Self::Item> {
        let headers = &self.view.log.headers;
        let filter = &self.view.filter;

        let next = get_next(
            self.index,
            filter,
            |index| name_unit_into(headers.main_frames.get(index).unwrap()),
            |index| name_unit_into(headers.slow_frames.get(index).unwrap()),
            |index| {
                let def = headers.gps_frames.as_ref().unwrap();
                name_unit_into(def.get(index).unwrap())
            },
        )?;

        self.index += 1;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.view.field_count() - self.index;
        (len, Some(len))
    }
}

impl<'a, V> FusedIterator for FieldIter<'a, V> where Self: Iterator {}
impl<'a, V> ExactSizeIterator for FieldIter<'a, V> where Self: Iterator {}

#[derive(Debug)]
pub struct FrameIter<'a, V> {
    view: &'a V,
    len: usize,
    index: usize,
}

impl<'a, V> FrameIter<'a, V> {
    const fn new(view: &'a V, len: usize) -> Self {
        Self {
            view,
            len,
            index: 0,
        }
    }
}

impl<'view: 'log, 'log: 'data, 'data> Iterator for FrameIter<'view, MainView<'log, 'data>> {
    type Item = FieldValueIter<'view, MainView<'log, 'data>>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        (index < self.len).then(|| {
            self.index += 1;
            FieldValueIter::new(self.view, index)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.view.frame_count() - self.index;
        (len, Some(len))
    }
}

impl<'a, V> FusedIterator for FrameIter<'a, V> where Self: Iterator {}
impl<'a, V> ExactSizeIterator for FrameIter<'a, V> where Self: Iterator {}

#[derive(Debug)]
pub struct FieldValueIter<'a, V> {
    view: &'a V,
    frame: usize,
    field: usize,
}

impl<'a, V> FieldValueIter<'a, V> {
    const fn new(view: &'a V, frame: usize) -> Self {
        Self {
            view,
            frame,
            field: 0,
        }
    }
}

impl<'view: 'log, 'log: 'data, 'data> Iterator for FieldValueIter<'view, MainView<'log, 'data>> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let MainView {
            log: Log { headers, data, .. },
            filter,
        } = &self.view;

        let FrameSync { main, slow, gps } = &data.main_frames[self.frame];

        let next = get_next(
            self.field,
            filter,
            |index| main.get(index, headers).unwrap().into(),
            |index| data.slow_frames[*slow].values[index].into(),
            |index| data.gps_frames[gps.unwrap()].get(index).unwrap().into(),
        )?;

        self.field += 1;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.view.field_count() - self.field;
        (len, Some(len))
    }
}

impl<'a, V> FusedIterator for FieldValueIter<'a, V> where Self: Iterator {}
impl<'a, V> ExactSizeIterator for FieldValueIter<'a, V> where Self: Iterator {}

fn name_unit_into<T: Into<Unit>>((name, unit): (&str, T)) -> (&str, Unit) {
    (name, unit.into())
}

// Reason: lint ignores let-else
#[allow(unreachable_code)]
fn intersection(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut new = Vec::with_capacity(a.len().min(b.len()));
    let mut a = a.iter().peekable();
    let mut b = b.iter().peekable();

    loop {
        let (Some(next_a), Some(next_b)) = (a.peek(), b.peek()) else { return new; };

        match next_a.cmp(next_b) {
            Ordering::Less => {
                a.next();
            }
            Ordering::Equal => {
                new.push(**next_a);
                a.next();
                b.next();
            }
            Ordering::Greater => {
                b.next();
            }
        }
    }

    new
}

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

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use test_case::case;

    use super::*;

    #[case(Vec::new(), Vec::new() ; "both")]
    #[case(vec![0], Vec::new() ; "left")]
    #[case(Vec::new(), vec![0] ; "right")]
    fn intersection_empty(left: Vec<usize>, right: Vec<usize>) {
        let result = dbg!(intersection(&left, &right));
        assert!(result.is_empty());
    }

    #[case(vec![0, 1, 2], vec![0, 3] => vec![0] ; "left")]
    #[case(vec![0, 2], vec![1, 2, 3] => vec![2] ; "right")]
    fn intersection_skip(left: Vec<usize>, right: Vec<usize>) -> Vec<usize> {
        intersection(&left, &right)
    }
}
