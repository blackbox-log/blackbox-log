use alloc::vec::Vec;

use crate::headers_v2::frame_defs::{FrameDef, FrameDefs};
use crate::Reader;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {}

#[derive(Debug)]
pub struct DataParser<'data> {
    data: Reader<'data>,
    frames: FrameDefs<'data>,

    main: Vec<u32>,
    slow: Vec<u32>,
    gps: Vec<u32>,
    gps_home: Vec<u32>,
}

impl<'data> DataParser<'data> {
    pub fn new(data: &'data [u8], frames: FrameDefs<'data>) -> Self {
        Self {
            data: Reader::new(data),
            main: alloc::vec![0; frames.main.len()],
            slow: alloc::vec![0; frames.slow.len()],
            gps: alloc::vec![0; frames.gps.as_ref().map_or(0, FrameDef::len)],
            gps_home: alloc::vec![0; frames.gps_home.as_ref().map_or(0, FrameDef::len)],
            frames,
        }
    }

    pub fn next<'a, V: Visitor<'a>>(
        &'a mut self,
        mut visitor: V,
    ) -> Option<Result<V::Output, Error>> {
        self.data.skip_until_any(b"IPSGHE");
        let kind = self.data.read_u8()?;

        Some(match kind {
            b'I' => {
                let frame = self.frames.main.as_intra();
                decode_frame(&mut self.data, frame, &mut self.main)
                    .map(|raw| visitor.main(MainFrameKind::Intra, raw))
            }
            b'P' => {
                let frame = self.frames.main.as_inter();
                decode_frame(&mut self.data, frame, &mut self.main)
                    .map(|raw| visitor.main(MainFrameKind::Inter, raw))
            }
            b'S' => decode_frame(&mut self.data, &self.frames.slow, &mut self.slow)
                .map(|raw| visitor.slow(raw)),
            b'G' => {
                let Some(frame) = self.frames.gps.as_ref() else {
                    todo!()
                };
                decode_frame(&mut self.data, frame, &mut self.gps).map(|raw| visitor.gps(raw))
            }
            b'H' => {
                let Some(frame) = self.frames.gps_home.as_ref() else {
                    todo!()
                };
                decode_frame(&mut self.data, frame, &mut self.gps_home)
                    .map(|raw| visitor.gps_home(raw))
            }
            b'E' => Ok(visitor.event()),
            _ => unreachable!(),
        })
    }
}

pub trait Visitor<'a> {
    type Output;

    fn main(&mut self, kind: MainFrameKind, frame: &'a [u32]) -> Self::Output;
    fn slow(&mut self, frame: &'a [u32]) -> Self::Output;
    fn gps(&mut self, frame: &'a [u32]) -> Self::Output;
    fn gps_home(&mut self, frame: &'a [u32]) -> Self::Output;
    fn event(&mut self) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainFrameKind {
    Intra,
    Inter,
}

fn decode_frame<'a, F: FrameDef>(
    data: &mut Reader,
    frame: F,
    raw: &'a mut [u32],
) -> Result<&'a mut [u32], Error> {
    let mut encodings = frame.encodings().peekable();

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
        encoding.decode_into(data, &mut raw[i..next_i]).unwrap();
        i = next_i;
    }

    Ok(raw)
}
