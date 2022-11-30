#[cfg(not(any(fuzzing, bench)))]
pub(crate) mod decode;
#[cfg(any(fuzzing, bench))]
pub mod decode;

pub(crate) use self::decode::Encoding;

pub(crate) type InternalResult<T> = Result<T, InternalError>;

#[derive(Debug, Clone)]
pub(crate) enum InternalError {
    /// Found something unexpected, try to recover
    Retry,
    Eof,
}

pub(crate) fn to_base_field(field: &str) -> &str {
    field.split_once('[').map_or(field, |(base, _)| base)
}
