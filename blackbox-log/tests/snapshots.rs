use std::fs;
use std::io::Read;

use blackbox_log::parser::{Event, Headers, Stats, Value};
use blackbox_log::units::FlagSet;
use blackbox_log::Log;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

macro_rules! run {
    () => {
        |path| {
            let mut file = fs::File::open(path).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();

            let file = blackbox_log::File::new(&data);
            let logs = file
                .parse_iter()
                .map(|r| r.map(LogSnapshot::from))
                .collect::<Vec<_>>();

            insta::assert_ron_snapshot!(logs);
        }
    };
}

#[test]
fn own() {
    insta::glob!("logs/*.bbl", run!());
}

#[test]
fn fc_blackbox() {
    insta::glob!("logs/fc-blackbox/*", run!());
}

#[test]
fn gimbal_ghost() {
    insta::glob!("logs/gimbal-ghost/*", run!());
}

#[derive(Debug, Serialize)]
struct LogSnapshot<'a> {
    headers: Headers<'a>,
    stats: Stats,
    events: Vec<Event>,
    fields: Fields,
}

impl<'a> From<Log<'a>> for LogSnapshot<'a> {
    fn from(log: Log<'a>) -> Self {
        let fields = log.iter_fields().map(|(name, _)| name).collect::<Fields>();

        let fields = log.iter_frames().fold(fields, |mut fields, frame| {
            fields.update(frame.map(value_to_int));
            fields
        });

        Self {
            headers: log.headers().clone(),
            stats: log.stats(),
            events: log.events().to_owned(),
            fields,
        }
    }
}

#[derive(Debug, Serialize)]
struct Fields(Vec<FieldSnapshot>);

impl Fields {
    fn update(&mut self, frame: impl Iterator<Item = i128>) {
        for (field, value) in self.0.iter_mut().zip(frame) {
            field.update(value);
        }
    }
}

impl<T> FromIterator<T> for Fields
where
    T: Into<String>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(FieldSnapshot::new).collect())
    }
}

#[derive(Debug)]
struct FieldSnapshot {
    name: String,
    min: i128,
    max: i128,
    seen: Vec<i128>,
    histogram: [i128; 16],
}

impl FieldSnapshot {
    fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            min: 0,
            max: 0,
            seen: Vec::new(),
            histogram: [0; 16],
        }
    }

    fn update(&mut self, value: i128) {
        if value < self.min {
            self.min = value;
        } else if value > self.max {
            self.max = value;
        }

        // Insert new value if not in seen, keeping seen sorted
        let index = self.seen.partition_point(|&x| x < value);
        if self.seen.get(index) != Some(&value) {
            self.seen.insert(index, value);
        }

        // Group into buckets using the bottom bits since those vary the most
        let index = (value.unsigned_abs() % 16) as usize;
        self.histogram[index] += 1;
    }
}

impl Serialize for FieldSnapshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let unique = self.seen.len();

        let mut state =
            serializer.serialize_struct("FieldSnapshot", if unique > 1 { 5 } else { 4 })?;

        state.serialize_field("name", &self.name)?;
        state.serialize_field("min", &self.min)?;
        state.serialize_field("max", &self.max)?;
        state.serialize_field("unique", &unique)?;

        if unique > 1 {
            state.serialize_field("histogram", &self.histogram)?;
        }

        state.end()
    }
}

fn value_to_int(value: Value) -> i128 {
    match value {
        Value::FrameTime(x) => x.into(),
        Value::Amperage(x) => x.as_raw().into(),
        Value::Voltage(x) => x.as_raw().into(),
        Value::Acceleration(x) => x.as_raw().into(),
        Value::Rotation(x) => x.as_raw().into(),
        Value::FlightMode(x) => x.as_raw().into(),
        Value::State(x) => x.as_raw().into(),
        Value::FailsafePhase(x) => x.as_raw().into(),
        Value::Boolean(x) => x.into(),
        Value::Unsigned(x) => x.into(),
        Value::Signed(x) => x.into(),
    }
}
