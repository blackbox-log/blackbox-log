use alloc::vec::Vec;

use num_enum::TryFromPrimitive;
use tracing::instrument;

use crate::parser::reader::ByteReader;
use crate::parser::{decode, ParseError, ParseResult, Reader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    SyncBeep(u64),
    Disarm(u32),
    FlightMode { flags: u32, last_flags: u32 },
    End,
}

impl Event {
    #[instrument(level = "debug", name = "Event::parse", skip_all, fields(kind))]
    pub fn parse_into(data: &mut Reader, events: &mut Vec<Self>) -> ParseResult<bool> {
        let kind = data
            .bytes()
            .read_u8()
            .map(EventKind::try_from)
            .ok_or(ParseError::UnexpectedEof)?;

        match kind {
            Ok(EventKind::SyncBeep) => {
                // TODO: SyncBeep handle time rollover

                let time = decode::variable(data)?;
                events.push(Self::SyncBeep(time.into()));
                Ok(false)
            }

            Ok(EventKind::Disarm) => {
                let reason = decode::variable(data)?;
                events.push(Self::Disarm(reason));
                Ok(false)
            }

            Ok(EventKind::FlightMode) => {
                let flags = decode::variable(data)?;
                let last_flags = decode::variable(data)?;
                events.push(Self::FlightMode { flags, last_flags });
                Ok(false)
            }

            Ok(EventKind::End) => {
                let mut bytes = data.bytes();

                check_message(&mut bytes, b"End of log")?;

                if bytes.peek() == Some(b' ') {
                    // Assume INAV's new format:
                    // `End of log (disarm reason:x)\0`

                    check_message(&mut bytes, b" (disarm reason:")?;

                    let reason = bytes.read_u8().ok_or(ParseError::UnexpectedEof)?;
                    events.push(Self::Disarm(reason.into()));

                    if bytes.read_u8() != Some(b')') {
                        return Err(ParseError::Corrupted);
                    }
                }

                if bytes.read_u8() != Some(0) {
                    return Err(ParseError::Corrupted);
                }

                events.push(Self::End);
                Ok(true)
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

fn check_message(bytes: &mut ByteReader, message: &[u8]) -> ParseResult<()> {
    let bytes = bytes.read_n_bytes(message.len());

    if bytes.len() != message.len() {
        return Err(ParseError::UnexpectedEof);
    }

    if bytes != message {
        return Err(ParseError::Corrupted);
    }

    Ok(())
}
