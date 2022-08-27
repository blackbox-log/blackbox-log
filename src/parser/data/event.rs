use crate::{encoding, ParseError, ParseResult};
use biterator::Biterator;
use num_enum::TryFromPrimitive;
use std::io::Read;
use tracing::instrument;

pub type Time = u64;
pub type DisarmReason = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    SyncBeep(Time),
    Disarm(DisarmReason),
    End,
}

impl Event {
    #[instrument(level = "debug", name = "Event::parse", skip_all, fields(kind))]
    pub fn parse<R>(data: &mut Biterator<R>) -> ParseResult<Self>
    where
        R: Read,
    {
        let kind = data.next_byte().map(EventKind::try_from);
        match kind {
            Some(Ok(EventKind::SyncBeep)) => {
                // TODO: SyncBeep handle time rollover

                let time = encoding::read_uvar(data)?;
                Ok(Self::SyncBeep(time.into()))
            }
            Some(Ok(EventKind::Disarm)) => {
                let reason = encoding::read_uvar(data)?;
                Ok(Self::Disarm(reason))
            }
            Some(Ok(EventKind::End)) => {
                const END_MESSAGE: &str = "End of log\0";

                if !data.bytes().take(11).eq(END_MESSAGE.bytes()) {
                    todo!("malformed end event");
                }

                Ok(Self::End)
            }
            Some(Ok(event)) => todo!("unsupported event: {:?}", event),
            Some(Err(err)) => todo!("invalid event: {err}"),
            None => Err(ParseError::unexpected_eof()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
enum EventKind {
    SyncBeep = 0,
    InflightAdjustment = 13,
    Resume = 14,
    Disarm = 15,
    FlightMode = 40,
    End = 255,
}
