//! Minimal set of imports for convenience.
//!
//! [`File`](`crate::File`) is not included due to avoid collisions with other
//! types.

pub use crate::frame::{Frame as _, FrameDef as _};
pub use crate::units::{Flag as _, FlagSet as _};
pub use crate::{DataParser, Headers, ParserEvent};
