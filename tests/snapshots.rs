use std::collections::BTreeMap;
use std::fs;
use std::io::Read;

use blackbox_log::data::Stats;
use blackbox_log::frame::{self, FieldDef, GpsFrameDef};
use blackbox_log::prelude::*;
use blackbox_log::units::si;
use blackbox_log::{headers, Event, Unit, Value};
use hashbrown::HashMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

const MAX_EVENTS: usize = 200_000;
const MAX_LOGS: usize = 2;

macro_rules! run {
    () => {
        |path| {
            let mut file = fs::File::open(path).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();

            let file = blackbox_log::File::new(&data);
            let snapshot = FileSnapshot::from(file);

            insta::assert_ron_snapshot!(snapshot);
        }
    };
}

#[test]
fn own() {
    insta::glob!("logs/*.bbl", run!());
}

#[test]
#[ignore]
fn fc_blackbox() {
    insta::glob!("logs/fc-blackbox/*", run!());
}

#[test]
#[ignore]
fn gimbal_ghost() {
    insta::glob!("logs/gimbal-ghost/*", run!());
}

#[derive(Debug, Serialize)]
#[serde(rename = "File")]
struct FileSnapshot<'data> {
    count: usize,
    logs: Vec<headers::ParseResult<LogSnapshot<'data>>>,
}

impl<'data> From<blackbox_log::File<'data>> for FileSnapshot<'data> {
    fn from(file: blackbox_log::File<'data>) -> Self {
        let logs = file
            .iter()
            .map(|mut reader| {
                Headers::parse(&mut reader).map(|headers| {
                    let data = DataParser::new(reader, &headers);
                    LogSnapshot::new(&headers, data)
                })
            })
            .take(MAX_LOGS)
            .collect::<Vec<_>>();

        Self {
            count: file.log_count(),
            logs,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "Log")]
struct LogSnapshot<'data> {
    headers: HeadersSnapshot<'data>,
    stats: Stats,
    capped: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    events: Vec<Event>,
    #[serde(skip_serializing_if = "MainSnapshot::is_empty")]
    main: MainSnapshot,
    #[serde(skip_serializing_if = "SlowSnapshot::is_empty")]
    slow: SlowSnapshot,
    #[serde(skip_serializing_if = "GpsSnapshot::is_empty")]
    gps: GpsSnapshot,
}

impl<'data> LogSnapshot<'data> {
    fn new(headers: &Headers<'data>, mut data: DataParser<'data, '_>) -> Self {
        let mut events = Vec::new();

        let main = headers.main_frame_def.iter().collect::<Fields>();
        let mut main = MainSnapshot::new(main);

        let slow = headers.slow_frame_def.iter().collect::<Fields>();
        let mut slow = SlowSnapshot::new(slow);

        let gps = headers
            .gps_frame_def
            .iter()
            .flat_map(GpsFrameDef::iter)
            .collect::<Fields>();
        let mut gps = GpsSnapshot::new(gps);

        let mut capped = true;
        for _ in 0..MAX_EVENTS {
            let Some(frame) = data.next() else {
                capped = false;
                break;
            };

            match frame {
                ParserEvent::Event(event) => events.push(event),
                ParserEvent::Main(frame) => main.update(frame),
                ParserEvent::Slow(frame) => slow.update(frame),
                ParserEvent::Gps(frame) => gps.update(frame),
            }
        }

        Self {
            headers: headers.into(),
            stats: data.stats().clone(),
            capped,
            events,
            main,
            slow,
            gps,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "Headers")]
struct HeadersSnapshot<'data> {
    firmware_revision: &'data str,
    firmware: headers::Firmware,
    firmware_date: Option<Result<String, &'data str>>,
    board_info: Option<&'data str>,
    craft_name: Option<&'data str>,
    debug_mode: headers::DebugMode,
    disabled_fields: headers::DisabledFields,
    features: headers::FeatureSet,
    unknown: HashMap<&'data str, &'data str>,
}

impl<'data, 'a> From<&'a Headers<'data>> for HeadersSnapshot<'data> {
    fn from(headers: &'a Headers<'data>) -> Self {
        HeadersSnapshot {
            firmware_revision: headers.firmware_revision(),
            firmware: headers.firmware(),
            firmware_date: headers
                .firmware_date()
                .map(|res| res.map(|date| date.to_string())),
            board_info: headers.board_info(),
            craft_name: headers.craft_name(),
            debug_mode: headers.debug_mode(),
            disabled_fields: headers.disabled_fields(),
            features: headers.features(),
            unknown: headers.unknown().clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "MainFrame")]
struct MainSnapshot {
    count: u128,
    last_iteration: u32,
    time: NumberHistory<16, i128>,
    fields: Fields,
}

impl MainSnapshot {
    fn new(fields: Fields) -> Self {
        Self {
            count: 0,
            last_iteration: 0,
            time: NumberHistory::new(),
            fields,
        }
    }

    fn update(&mut self, frame: frame::MainFrame) {
        self.count += 1;
        self.last_iteration = frame.iteration();
        self.time.update(frame.time_raw().into());
        self.fields.update(frame);
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "SlowFrame")]
struct SlowSnapshot {
    count: u32,
    fields: Fields,
}

impl SlowSnapshot {
    fn new(fields: Fields) -> Self {
        Self { count: 0, fields }
    }

    fn update(&mut self, frame: frame::SlowFrame) {
        self.count += 1;
        self.fields.update(frame);
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "GpsFrame")]
struct GpsSnapshot {
    count: u32,
    time: (),
    fields: Fields,
}

impl GpsSnapshot {
    fn new(fields: Fields) -> Self {
        Self {
            count: 0,
            time: (),
            fields,
        }
    }

    fn update(&mut self, frame: frame::GpsFrame) {
        self.count += 1;
        self.fields.update(frame);
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Serialize)]
struct Fields(Vec<FieldSnapshot>);

impl Fields {
    fn update<F: blackbox_log::frame::Frame>(&mut self, frame: F) {
        for (field, value) in self.0.iter_mut().zip(frame.iter()) {
            field.update(value.into());
        }
    }
}

impl<'data, U: Into<Unit>> FromIterator<FieldDef<'data, U>> for Fields {
    fn from_iter<I: IntoIterator<Item = FieldDef<'data, U>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|field| FieldSnapshot::new(field.name.to_owned(), field.unit.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "Field")]
struct FieldSnapshot {
    name: String,
    unit: Unit,
    history: History,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum History {
    Int(NumberHistory<16, i128>),
    Bool { yes: usize, no: usize },
    Flags(BTreeMap<&'static str, usize>),
}

#[derive(Debug)]
struct NumberHistory<const N: usize, T> {
    min: T,
    max: T,
    seen: Vec<T>,
    histogram: [usize; N],
}

impl FieldSnapshot {
    fn new(name: String, unit: Unit) -> Self {
        Self {
            name,
            unit,
            history: match unit {
                Unit::Amperage
                | Unit::Voltage
                | Unit::Acceleration
                | Unit::Rotation
                | Unit::GpsCoordinate
                | Unit::Altitude
                | Unit::Velocity
                | Unit::GpsHeading
                | Unit::Unitless => History::Int(NumberHistory::new()),
                Unit::Boolean => History::Bool { yes: 0, no: 0 },
                Unit::FlightMode | Unit::State | Unit::FailsafePhase => {
                    History::Flags(BTreeMap::new())
                }
            },
        }
    }

    #[allow(clippy::wildcard_enum_match_arm, clippy::cast_possible_truncation)]
    fn update(&mut self, value: Value) {
        match &mut self.history {
            History::Int(history) => history.update(match value {
                Value::Amperage(a) => a.get::<si::electric_current::milliampere>().round() as i128,
                Value::Voltage(v) => v.get::<si::electric_potential::millivolt>().round() as i128,
                Value::Acceleration(a) => a
                    .get::<si::acceleration::centimeter_per_second_squared>()
                    .round() as i128,
                Value::Rotation(r) => {
                    r.get::<si::angular_velocity::degree_per_second>().round() as i128
                }
                Value::GpsCoordinate(c) => (c * 10000000.).round() as i128,
                Value::Altitude(a) => a.get::<si::length::meter>().round() as i128,
                Value::Velocity(v) => {
                    v.get::<si::velocity::centimeter_per_second>().round() as i128
                }
                Value::GpsHeading(h) => (h * 10.).round() as i128,
                Value::Unsigned(u) => u.into(),
                Value::Signed(s) => s.into(),
                _ => unreachable!(),
            }),
            History::Bool { yes, no } => match value {
                Value::Boolean(true) => *yes += 1,
                Value::Boolean(false) => *no += 1,
                _ => unreachable!(),
            },
            History::Flags(history) => {
                if let Value::FailsafePhase(phase) = value {
                    *history.entry(phase.as_name()).or_insert(0) += 1;
                } else {
                    let flags = match value {
                        Value::FlightMode(m) => m.as_names(),
                        Value::State(s) => s.as_names(),
                        _ => unreachable!(),
                    };

                    for flag in flags {
                        *history.entry(flag).or_insert(0) += 1;
                    }
                }
            }
        }
    }
}

impl<const N: usize, T> NumberHistory<N, T> {
    fn new() -> Self
    where
        T: Default,
    {
        Self {
            min: T::default(),
            max: T::default(),
            seen: Vec::new(),
            histogram: [0; N],
        }
    }

    fn update_range(&mut self, value: T)
    where
        T: PartialOrd,
    {
        if value < self.min {
            self.min = value;
        } else if value > self.max {
            self.max = value;
        }
    }

    /// Store new value sorted in `seen` if not already present
    fn update_seen(&mut self, value: T)
    where
        T: PartialOrd,
    {
        let index = self.seen.partition_point(|x| *x < value);
        if self.seen.get(index) != Some(&value) {
            self.seen.insert(index, value);
        }
    }

    /// Bucket so that sequential values go in sequential buckets, since data is
    /// usually clustered
    #[inline(always)]
    fn update_histogram(&mut self, value: u128) {
        self.histogram[(value % N as u128) as usize] += 1;
    }
}

impl<const N: usize> NumberHistory<N, i128> {
    fn update(&mut self, value: i128) {
        self.update_range(value);
        self.update_seen(value);
        self.update_histogram(value.unsigned_abs());
    }
}

impl<const N: usize> Serialize for NumberHistory<N, i128>
where
    [usize; N]: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(self, self.min, self.max, serializer)
    }
}

impl<const N: usize> Serialize for NumberHistory<N, f64>
where
    [usize; N]: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(
            self,
            format!("{:.2}", self.min),
            format!("{:.2}", self.max),
            serializer,
        )
    }
}

#[inline(always)]
fn serialize<const N: usize, T, S, U>(
    snapshot: &NumberHistory<N, T>,
    min: U,
    max: U,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    U: Serialize,
    [usize; N]: Serialize,
{
    let unique = snapshot.seen.len();

    let mut state = serializer.serialize_struct("FieldSnapshot", 4)?;

    state.serialize_field("min", &min)?;
    state.serialize_field("max", &max)?;
    state.serialize_field("unique", &unique)?;

    let histogram = "histogram";
    if unique > 1 {
        state.serialize_field(histogram, &snapshot.histogram)?;
    } else {
        state.skip_field(histogram)?;
    }

    state.end()
}
