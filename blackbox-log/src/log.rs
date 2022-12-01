//! [`Log`] and views into it.

use alloc::vec::Vec;
use core::cmp::Ordering;
use core::iter::FusedIterator;

use crate::data::{Data, FrameSync, Stats};
use crate::event::Event;
use crate::frame::{GpsUnit, GpsValue};
use crate::parser::to_base_field;
use crate::{Headers, HeadersParseResult, Reader, Unit, Value};

/// A single blackbox log.
#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
}

impl<'data> Log<'data> {
    /// Attempts to fully parse a single blackbox log.
    ///
    /// **Note:** This assumes that `data` is already aligned to the beginning
    /// of the log.
    pub fn parse(data: &mut Reader<'data>) -> HeadersParseResult<Self> {
        let headers = Headers::parse(data)?;
        let data = Data::parse(data, &headers);

        Ok(Self { headers, data })
    }

    /// Attempts to parse a single blackbox log using pre-parsed `Headers`.
    ///
    /// **Note:** This assumes that `data` is already aligned to the beginning
    /// of the data section of the log.
    pub fn parse_with_headers(data: &mut Reader<'data>, headers: Headers<'data>) -> Self {
        let data = Data::parse(data, &headers);
        Self { headers, data }
    }

    /// Returns the parsed headers.
    pub const fn headers(&self) -> &Headers<'data> {
        &self.headers
    }

    /// Returns the parsed events.
    pub fn events(&self) -> &[Event] {
        &self.data.events
    }

    /// Returns statistics about the log.
    pub fn stats(&self) -> Stats {
        self.data.to_stats()
    }

    /// Returns a [`LogView`] that iterates over main frames and *does not*
    /// include any data from GPS frames.
    pub fn data<'log>(&'log self) -> MainView<'log, 'data> {
        let mut filter = Filter::new_unfiltered(&self.headers);
        filter.gps = Vec::new();
        MainView { log: self, filter }
    }

    /// Returns a [`LogView`] that iterates over main frames and *does* include
    /// data from GPS frames.
    ///
    /// **Note:** This does mean that GPS data will likely be repeated, as main
    /// frames are usually logged much more frequently than GPS frames. If there
    /// are multiple GPS frames logged between main frames, only the last will
    /// be included.
    pub fn merged_data<'log>(&'log self) -> MainView<'log, 'data> {
        let mut filter = Filter::new_unfiltered(&self.headers);

        // Filter out the GPS time field
        if self.headers.gps_frames.is_some() {
            filter.gps.remove(0);
        }

        MainView { log: self, filter }
    }

    /// Returns a [`LogView`] that iterates over only GPS frames.
    pub fn gps_data<'log>(&'log self) -> GpsView<'log, 'data> {
        GpsView { log: self }
    }
}

/// Provides access to the parsed data section of a log.
pub trait LogView<'view, 'data>: 'view {
    type Unit;
    type Value;

    type FieldIter: Iterator<Item = (&'data str, Self::Unit)>;
    type FrameIter: Iterator<Item = Self::ValueIter>;
    type ValueIter: Iterator<Item = Self::Value>;

    /// Returns the number of fields included in the view.
    fn field_count(&'view self) -> usize;

    /// Returns the number of frames included in the view.
    fn frame_count(&'view self) -> usize;

    /// Returns an iterator over the fields included in the view and their
    /// units.
    fn fields(&'view self) -> Self::FieldIter;

    /// Returns an iterator over the frames included in the view. Each frame is
    /// an iterator over its values.
    fn values(&'view self) -> Self::FrameIter;
}

/// A [`LogView`] into a [`Log`]'s data that iterates over each main frame. See
/// [`Log::data`] and [`Log::merged_data`].
#[derive(Debug, Clone)]
pub struct MainView<'log: 'data, 'data> {
    log: &'log Log<'data>,
    filter: Filter,
}

impl MainView<'_, '_> {
    pub fn update_filter<S: AsRef<str>>(&mut self, filter: &[S]) {
        self.filter.merge(&Filter::new(filter, &self.log.headers));
    }
}

impl<'view: 'data, 'data> LogView<'view, 'data> for MainView<'_, 'data>
where
    Self: 'view,
{
    type FieldIter = FieldIter<'view, Self>;
    type FrameIter = FrameIter<'view, Self>;
    type Unit = Unit;
    type Value = Value;
    type ValueIter = FieldValueIter<'view, Self>;

    #[inline]
    fn field_count(&self) -> usize {
        self.filter.len()
    }

    #[inline]
    fn frame_count(&self) -> usize {
        self.log.data.main_frames.len()
    }

    #[inline]
    fn fields(&self) -> FieldIter<'_, Self> {
        FieldIter::new(self)
    }

    #[inline]
    fn values(&self) -> FrameIter<'_, Self> {
        FrameIter::new(self, self.frame_count())
    }
}

/// A [`LogView`] into a [`Log`]'s data that iterates over each GPS frames. See
/// [`Log::gps_data`].
#[derive(Debug, Clone)]
pub struct GpsView<'log: 'data, 'data> {
    log: &'log Log<'data>,
}

impl<'view: 'data, 'data> LogView<'view, 'data> for GpsView<'_, 'data>
where
    Self: 'view,
{
    type FieldIter = FieldIter<'view, Self>;
    type FrameIter = FrameIter<'view, Self>;
    type Unit = GpsUnit;
    type Value = GpsValue;
    type ValueIter = FieldValueIter<'view, Self>;

    // Reason: cannot name type of gps here
    #[allow(clippy::redundant_closure_for_method_calls)]
    #[inline]
    fn field_count(&self) -> usize {
        self.log
            .headers
            .gps_frames
            .as_ref()
            .map_or(0, |gps| gps.len())
    }

    #[inline]
    fn frame_count(&self) -> usize {
        self.log.data.gps_frames.len()
    }

    #[inline]
    fn fields(&self) -> FieldIter<'_, Self> {
        FieldIter::new(self)
    }

    #[inline]
    fn values(&self) -> FrameIter<'_, Self> {
        FrameIter::new(self, self.frame_count())
    }
}

/// An iterator over field name & unit pairs. See [`LogView::fields`].
#[derive(Debug)]
pub struct FieldIter<'v, V> {
    view: &'v V,
    index: usize,
}

impl<'v, V> FieldIter<'v, V> {
    const fn new(view: &'v V) -> Self {
        Self { view, index: 0 }
    }
}

impl<'data> Iterator for FieldIter<'_, MainView<'_, 'data>> {
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

impl<'data> Iterator for FieldIter<'_, GpsView<'_, 'data>> {
    type Item = (&'data str, GpsUnit);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.view.log.headers.gps_frames.as_ref()?.get(self.index)?;
        self.index += 1;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.view.field_count() - self.index;
        (len, Some(len))
    }
}

impl<V> FusedIterator for FieldIter<'_, V> where Self: Iterator {}
impl<V> ExactSizeIterator for FieldIter<'_, V> where Self: Iterator {}

/// An iterator over the data frames included in a [`LogView`]. See
/// [`LogView::values`].
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

impl<'v, 'd, V: LogView<'v, 'd>> Iterator for FrameIter<'v, V> {
    type Item = FieldValueIter<'v, V>;

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

impl<V> FusedIterator for FrameIter<'_, V> where Self: Iterator {}
impl<V> ExactSizeIterator for FrameIter<'_, V> where Self: Iterator {}

/// An iterator over the values in a single frame.
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

impl Iterator for FieldValueIter<'_, MainView<'_, '_>> {
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

impl Iterator for FieldValueIter<'_, GpsView<'_, '_>> {
    type Item = GpsValue;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.view.log.data.gps_frames[self.frame].get(self.field)?;
        self.field += 1;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.view.field_count() - self.field;
        (len, Some(len))
    }
}

impl<V> FusedIterator for FieldValueIter<'_, V> where Self: Iterator {}
impl<V> ExactSizeIterator for FieldValueIter<'_, V> where Self: Iterator {}

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
