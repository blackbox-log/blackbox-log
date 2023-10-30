use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::frame::FrameDef;
use crate::utils::to_base_field;

/// A complete set of filters ready to be passed to
/// [`Headers::data_parser_with_filters`][crate::Headers::data_parser_with_filters].
#[derive(Debug, Default, Clone)]
pub struct FilterSet {
    pub main: Filter,
    pub slow: Filter,
    pub gps: Filter,
}

/// A filter for the fields to include in one kind of frame.
#[derive(Debug, Clone, Default)]
pub enum Filter {
    /// Include all fields of this frame kind.
    #[default]
    Unfiltered,
    /// Include a subset of fields from this frame kind.
    ///
    /// **Note**: Any fields requested that are not present in the log will not
    /// be included.
    OnlyFields(FieldFilter),
}

/// A set of field names to include in one kind of frame.
#[derive(Debug, Clone)]
pub struct FieldFilter(HashSet<String>);

impl Filter {
    /// Only include any required fields (ie time for main and gps frames and
    /// none for slow frames).
    pub fn only_required() -> Self {
        Self::OnlyFields(FieldFilter(HashSet::new()))
    }

    pub(crate) fn apply<'data, F: FrameDef<'data>>(&self, frame: &F) -> AppliedFilter {
        match self {
            Filter::Unfiltered => AppliedFilter::new_unfiltered(frame.len()),
            Filter::OnlyFields(fields) => frame
                .iter()
                .enumerate()
                .filter_map(|(i, field)| fields.0.contains(field.name).then_some(i))
                .collect(),
        }
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

#[derive(Debug, Clone, Default)]
pub(crate) struct AppliedFilter(Vec<usize>);

impl AppliedFilter {
    fn new_unfiltered(len: usize) -> Self {
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
