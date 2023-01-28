use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::parser::to_base_field;

/// A filter for the fields to be included in one kind of frame.
///
/// See [`FrameDef::apply_filter`][`crate::frame::FrameDef::apply_filter`].
#[derive(Debug, Clone)]
pub struct FieldFilter(HashSet<String>);

impl FieldFilter {
    #[allow(single_use_lifetimes)]
    pub(crate) fn apply<'a>(&self, fields: impl Iterator<Item = &'a str>) -> AppliedFilter {
        fields
            .enumerate()
            .filter_map(|(i, field)| self.0.contains(to_base_field(field)).then_some(i))
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
