use crate::parser::{decode, ParseError, ParseResult};
use crate::Reader;
use bitter::BitReader;
use num_enum::TryFromPrimitive;
use std::iter;
use tracing::instrument;

pub type Time = u64;
pub type DisarmReason = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    SyncBeep(Time),
    Disarm(DisarmReason),
    FlightMode { flags: u32, last_flags: u32 },
    End,
}

impl Event {
    #[instrument(level = "debug", name = "Event::parse", skip_all, fields(kind))]
    pub fn parse(data: &mut Reader) -> ParseResult<Self> {
        let kind = data
            .read_u8()
            .map(EventKind::try_from)
            .ok_or(ParseError::UnexpectedEof)?;

        match kind {
            Ok(EventKind::SyncBeep) => {
                // TODO: SyncBeep handle time rollover

                let time = decode::variable(data)?;
                Ok(Self::SyncBeep(time.into()))
            }

            Ok(EventKind::Disarm) => {
                let reason = decode::variable(data)?;
                Ok(Self::Disarm(reason))
            }

            Ok(EventKind::FlightMode) => {
                let flags = decode::variable(data)?;
                let last_flags = decode::variable(data)?;
                Ok(Self::FlightMode { flags, last_flags })
            }

            Ok(EventKind::End) => {
                const END_MESSAGE: &str = "End of log\0";

                if !iter::from_fn(|| data.read_u8())
                    .take(11)
                    .eq(END_MESSAGE.bytes())
                {
                    return Err(ParseError::Corrupted);
                }

                Ok(Self::End)
            }

            Ok(event) => todo!("unsupported event: {:?}", event),
            Err(err) => todo!("invalid event: {err}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
enum EventKind {
    SyncBeep = 0,
    AutotuneCycleStart = 10,
    AutotuneCycleResult = 11,
    AutotuneTargets = 12,
    InflightAdjustment = 13,
    Resume = 14,
    Disarm = 15,
    GTuneCycleResult = 20,
    FlightMode = 30,
    End = 255,
}
