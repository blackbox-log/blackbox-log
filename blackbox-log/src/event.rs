use alloc::vec::Vec;

use tracing::instrument;

use crate::parser::{decode, InternalError, InternalResult};
use crate::Reader;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Event {
    SyncBeep(u64),
    InflightAdjustment {
        function: u8,
        new_value: AdjustedValue,
    },
    Resume {
        log_iteration: u32,
        time: u32,
    },
    Disarm(u32),
    FlightMode {
        flags: u32,
        last_flags: u32,
    },
    ImuFailure {
        error: u32,
    },
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AdjustedValue {
    Float(f32),
    Int(i32),
}

impl Event {
    #[instrument(level = "debug", name = "Event::parse", skip_all, fields(kind))]
    pub(crate) fn parse_into(
        data: &mut Reader,
        events: &mut Vec<Self>,
    ) -> InternalResult<EventKind> {
        let byte = data.read_u8().ok_or(InternalError::Eof)?;
        let kind = EventKind::from_byte(byte).ok_or_else(|| {
            tracing::debug!("found invalid event: {byte:0>#2x}");
            InternalError::Retry
        })?;

        match kind {
            EventKind::SyncBeep => {
                // TODO: SyncBeep handle time rollover

                let time = decode::variable(data)?;
                events.push(Self::SyncBeep(time.into()));
            }

            EventKind::InflightAdjustment => {
                let function = data.read_u8().ok_or(InternalError::Eof)?;

                let new_value = if (function & 0x80) > 0 {
                    AdjustedValue::Float(data.read_f32().ok_or(InternalError::Eof)?)
                } else {
                    AdjustedValue::Int(decode::variable_signed(data)?)
                };

                events.push(Self::InflightAdjustment {
                    function: function & 0x7F,
                    new_value,
                });
            }

            EventKind::Resume => {
                let log_iteration = decode::variable(data)?;
                let time = decode::variable(data)?;

                events.push(Self::Resume {
                    log_iteration,
                    time,
                });
            }

            EventKind::Disarm => {
                let reason = decode::variable(data)?;
                events.push(Self::Disarm(reason));
            }

            EventKind::FlightMode => {
                let flags = decode::variable(data)?;
                let last_flags = decode::variable(data)?;
                events.push(Self::FlightMode { flags, last_flags });
            }

            EventKind::ImuFailure => {
                let error = decode::variable(data)?;
                events.push(Self::ImuFailure { error });
            }

            EventKind::End => {
                check_message(data, b"End of log")?;

                if data.peek() == Some(b' ') {
                    // Assume INAV's new format:
                    // `End of log (disarm reason:x)\0`

                    check_message(data, b" (disarm reason:")?;

                    let reason = data.read_u8().ok_or(InternalError::Eof)?;
                    events.push(Self::Disarm(reason.into()));

                    if data.read_u8() != Some(b')') {
                        return Err(InternalError::Retry);
                    }
                }

                if data.read_u8() != Some(0) {
                    return Err(InternalError::Retry);
                }

                events.push(Self::End);
            }
        }

        Ok(kind)
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub(crate) enum EventKind {
        SyncBeep = 0,
        InflightAdjustment = 13,
        Resume = 14,
        Disarm = 15,
        FlightMode = 30,
        ImuFailure = 40,
        End = 255,
    }
}

fn check_message(bytes: &mut Reader, message: &[u8]) -> InternalResult<()> {
    let bytes = bytes.read_n_bytes(message.len());

    if bytes.len() != message.len() {
        return Err(InternalError::Eof);
    }

    if bytes != message {
        return Err(InternalError::Retry);
    }

    Ok(())
}