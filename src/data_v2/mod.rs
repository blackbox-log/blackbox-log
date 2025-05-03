pub mod context;
mod history;

use alloc::vec::Vec;

pub use self::context::Context;
use self::history::FrameHistory;
pub(crate) use self::history::History;
use crate::headers_v2::frame_defs::{
    Field as _, FieldDetails as _, FrameDef, GpsFrameDef, GpsHomeFrameDef, Kind, MainFrameDef,
    SlowFrameDef,
};
use crate::predictor::PredictorContextV2;
use crate::Reader;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {}

#[derive(Debug)]
pub struct DataParser<'data> {
    data: Reader<'data>,
    context: Context<'data>,

    main: FrameHistory,
    slow: FrameHistory,
    gps: FrameHistory,
    gps_home: FrameHistory,
}

impl<'data> DataParser<'data> {
    pub fn new(data: &'data [u8], context: Context<'data>) -> Self {
        Self {
            data: Reader::new(data),

            main: FrameHistory::new(context.frames.main.len()),
            slow: FrameHistory::new(context.frames.slow.len()),
            gps: FrameHistory::new(context.frames.gps.as_ref().map_or(0, Vec::len)),
            gps_home: FrameHistory::new(context.frames.gps_home.as_ref().map_or(0, Vec::len)),

            context,
        }
    }

    pub fn next<'a, V: Visitor<'a>>(
        &'a mut self,
        mut visitor: V,
    ) -> Option<Result<V::Output, Error>> {
        self.data.skip_until_any(b"IPSGHE");
        let kind = self.data.read_u8()?;

        Some(match kind {
            b'I' => decode_frame(
                &mut self.data,
                self.context.frames.intra(),
                &mut self.context.predictor,
                &mut self.main,
            )
            .map(|frame| visitor.main(MainFrameKind::Intra, frame)),
            b'P' => decode_frame(
                &mut self.data,
                self.context.frames.inter(),
                &mut self.context.predictor,
                &mut self.main,
            )
            .map(|frame| visitor.main(MainFrameKind::Inter, frame)),
            b'S' => decode_frame(
                &mut self.data,
                self.context.frames.slow(),
                &mut self.context.predictor,
                &mut self.slow,
            )
            .map(|frame| visitor.slow(frame)),
            b'G' => {
                let Some(frame) = self.context.frames.gps() else {
                    todo!()
                };
                decode_frame(
                    &mut self.data,
                    frame,
                    &mut self.context.predictor,
                    &mut self.gps,
                )
                .map(|frame| visitor.gps(frame))
            }
            b'H' => {
                let Some(frame) = self.context.frames.gps_home() else {
                    todo!()
                };
                decode_frame(
                    &mut self.data,
                    frame,
                    &mut self.context.predictor,
                    &mut self.gps_home,
                )
                .map(|frame| visitor.gps_home(frame))
            }
            b'E' => Ok(visitor.event()),
            _ => unreachable!(),
        })
    }
}

pub trait Visitor<'a> {
    type Output;

    fn main(&mut self, kind: MainFrameKind, frame: Frame<'a, MainFrameDef>) -> Self::Output;
    fn slow(&mut self, frame: Frame<'a, SlowFrameDef>) -> Self::Output;
    fn gps(&mut self, frame: Frame<'a, GpsFrameDef>) -> Self::Output;
    fn gps_home(&mut self, frame: Frame<'a, GpsHomeFrameDef>) -> Self::Output;
    fn event(&mut self) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainFrameKind {
    Intra,
    Inter,
}

#[derive(Debug)]
pub struct Frame<'a, F> {
    def: F,
    raw: &'a [u32],
}

impl<F> Frame<'_, F>
where
    F: FrameDef + Copy,
{
    pub fn get(&self, index: usize) -> Option<u32> {
        // TODO:
        self.get_raw(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        (0..self.def.len()).map(|i| self.get(i).unwrap())
    }
}

impl<F> Frame<'_, F> {
    pub fn iter_raw(&self) -> impl Iterator<Item = u32> + '_ {
        self.raw.iter().copied()
    }

    pub fn get_raw(&self, index: usize) -> Option<u32> {
        self.raw.get(index).copied()
    }
}

fn decode_frame<'a, F: FrameDef>(
    data: &'a mut Reader,
    def: F,
    ctx: &'a mut PredictorContextV2,
    history: &'a mut FrameHistory,
) -> Result<Frame<'a, F>, Error> {
    let mut encodings = def.encodings().peekable();
    let mut i = 0;
    while let Some(encoding) = encodings.next() {
        let len = {
            let max = encoding.max_chunk_size();
            let mut len = 1;
            while len < max && encodings.next_if_eq(&encoding).is_some() {
                len += 1;
            }
            len
        };

        let next_i = i + len;
        encoding.decode_into(data, &mut history[i..next_i]).unwrap();
        i = next_i;
    }
    drop(encodings);

    for ((current, history), field) in history.iter().zip(def.iter()) {
        let skipped = 0; // FIXME
        *current = field
            .predictor()
            .apply_v2(*current, field.signed(), skipped, ctx, history);

        match (field.kind().into(), field.index()) {
            (Kind::Motor, Some(0)) => ctx.set_motor_0(*current),
            (Kind::HomeLatittude, _) => ctx.set_gps_home_lat(*current),
            (Kind::HomeLongitude, _) => ctx.set_gps_home_lon(*current),
            _ => {}
        }
    }

    let raw = history.finish();

    Ok(Frame { def, raw })
}
