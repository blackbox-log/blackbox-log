use super::PredictorContextV2;
use crate::headers_v2::frame_defs::{FrameDefBuilders, FrameDefs};

#[derive(Debug)]
pub struct Context<'data> {
    pub(super) frames: FrameDefs<'data>,
    pub(super) predictor: PredictorContextV2,
}

impl<'data> Context<'data> {
    pub const fn builder() -> Builder<'data> {
        Builder {
            frames: FrameDefBuilders::new(),
            predictor: PredictorContextV2::new(),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("failed to parse value")]
    InvalidValue,
    #[error(transparent)]
    FrameDefs(#[from] crate::headers_v2::frame_defs::ParseError),
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum BuildError {
    #[error(transparent)]
    FrameDefs(#[from] crate::headers_v2::frame_defs::BuildError),
}

#[derive(Debug)]
pub struct Builder<'data> {
    frames: FrameDefBuilders<'data>,
    predictor: PredictorContextV2,
}

impl<'data> Builder<'data> {
    pub fn update(&mut self, header: &'data str, value: &'data str) -> Result<(), Error> {
        self.frames.update(header, value)?;
        self.predictor
            .update(header, value)
            .map_err(|()| Error::InvalidValue)
    }

    pub fn build(self) -> Result<Context<'data>, BuildError> {
        Ok(Context {
            frames: self.frames.build()?,
            predictor: self.predictor,
        })
    }
}
