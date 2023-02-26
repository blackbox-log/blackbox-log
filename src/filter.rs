use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::frame::{self, FrameDef};
use crate::utils::to_base_field;

/// A complete set of filters to be passed to
/// [`DataParser::with_filters`][crate::DataParser::with_filters].
#[derive(Debug, Default, Clone)]
pub struct FieldFilterSet {
    pub main: Option<FieldFilter>,
    pub slow: Option<FieldFilter>,
    pub gps: Option<FieldFilter>,
}

impl FieldFilterSet {
    pub(crate) fn apply_main(&self, frame: &frame::MainFrameDef) -> AppliedFilter {
        self.main.as_ref().map_or_else(
            || AppliedFilter::new_unfiltered(frame.len()),
            |filter| filter.apply(frame),
        )
    }

    pub(crate) fn apply_slow(&self, frame: &frame::SlowFrameDef) -> AppliedFilter {
        self.slow.as_ref().map_or_else(
            || AppliedFilter::new_unfiltered(frame.len()),
            |filter| filter.apply(frame),
        )
    }

    pub(crate) fn apply_gps(&self, frame: Option<&frame::GpsFrameDef>) -> AppliedFilter {
        match (&self.gps, frame) {
            (Some(filter), Some(frame)) => filter.apply(frame),
            (None, Some(frame)) => AppliedFilter::new_unfiltered(frame.len()),
            _ => AppliedFilter::new_unfiltered(0),
        }
    }
}

/// A filter for the fields to be included in one kind of frame.
#[derive(Debug, Clone)]
pub struct FieldFilter(HashSet<String>);

impl FieldFilter {
    pub(crate) fn apply<'data, F: FrameDef<'data>>(&self, frame: &F) -> AppliedFilter {
        frame
            .iter()
            .enumerate()
            .filter_map(|(i, field)| self.0.contains(to_base_field(field.name)).then_some(i))
            .collect()
    }
}

impl<'a, S> From<&'a [S]> for FieldFilter
where
    &'a S: AsRef<str>,
{
    fn from(slice: &'a [S]) -> Self {
        slice.iter().collect()
    }
}

impl<S, const N: usize> From<[S; N]> for FieldFilter
where
    S: AsRef<str>,
{
    fn from(arr: [S; N]) -> Self {
        arr.into_iter().collect()
    }
}

impl<S> FromIterator<S> for FieldFilter
where
    S: AsRef<str>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let set = iter
            .into_iter()
            .map(|s| to_base_field(s.as_ref()).to_owned())
            .collect();
        Self(set)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AppliedFilter(Vec<usize>);

impl AppliedFilter {
    pub(crate) fn new_unfiltered(len: usize) -> Self {
        Self((0..len).collect())
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<usize> {
        self.0.get(index).copied()
    }
}

impl<T> FromIterator<T> for AppliedFilter
where
    Vec<usize>: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}
