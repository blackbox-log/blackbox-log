pub(crate) mod decode;

pub(crate) use self::decode::Encoding;

pub(crate) type InternalResult<T> = Result<T, InternalError>;

/// A recoverable error.
#[derive(Debug, Clone)]
pub(crate) enum InternalError {
    /// Found something unexpected, retry at the next byte.
    Retry,
    /// Found the end of file.
    Eof,
}
