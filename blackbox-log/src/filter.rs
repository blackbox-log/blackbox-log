use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::parser::to_base_field;

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

impl<S> FromIterator<S> for Filter
where
    S: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let set = iter.into_iter().map(Into::into).collect();
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().copied()
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
