// flightLogDisarmReason_e in core.h
generate_disarm_reason! {
    ArmingDisabled = 0,
    Failsafe = 1,
    ThrottleTimeout = 2,
    Sticks = 3,
    Switch = 4,
    CrashProtection = 5,
    RunawayTakeoff = 6,
    GpsRescue = 7,
    SerialCommand = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlightMode {
    Angle,
    Horizon,
    Mag,
    Baro,
    GpsHome,
    GpsHold,
    HeadFree,
    Passthru,
    RangeFinder,
    Failsafe,
}

impl FlightMode {
    const fn to_bit(self) -> usize {
        match self {
            Self::Angle => 0,
            Self::Horizon => 1,
            Self::Mag => 2,
            Self::Baro => 3,
            Self::GpsHome => 4,
            Self::GpsHold => 5,
            Self::HeadFree => 6,
            Self::Passthru => 8,
            Self::RangeFinder => 9,
            Self::Failsafe => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlightModeFlags(u32);

impl FlightModeFlags {
    pub const fn new(flags: u32) -> Self {
        Self(flags)
    }

    const fn is_bit_set(self, bit: usize) -> bool {
        (self.0 & (1 << bit)) > 0
    }

    pub const fn is_mode_set(self, mode: FlightMode) -> bool {
        self.is_bit_set(mode.to_bit())
    }

    pub const fn angle(self) -> bool {
        self.is_mode_set(FlightMode::Angle)
    }

    pub const fn horizon(self) -> bool {
        self.is_mode_set(FlightMode::Horizon)
    }

    pub const fn mag(self) -> bool {
        self.is_mode_set(FlightMode::Mag)
    }

    pub const fn baro(self) -> bool {
        self.is_mode_set(FlightMode::Baro)
    }

    pub const fn gps_home(self) -> bool {
        self.is_mode_set(FlightMode::GpsHome)
    }

    pub const fn gps_hold(self) -> bool {
        self.is_mode_set(FlightMode::GpsHold)
    }

    pub const fn headfree(self) -> bool {
        self.is_mode_set(FlightMode::HeadFree)
    }

    pub const fn passthru(self) -> bool {
        self.is_mode_set(FlightMode::Passthru)
    }

    pub const fn rangefinder(self) -> bool {
        self.is_mode_set(FlightMode::RangeFinder)
    }

    pub const fn failsafe(self) -> bool {
        self.is_mode_set(FlightMode::Failsafe)
    }

    pub fn to_modes(self) -> Vec<FlightMode> {
        [
            FlightMode::Angle,
            FlightMode::Horizon,
            FlightMode::Mag,
            FlightMode::Baro,
            FlightMode::GpsHome,
            FlightMode::GpsHold,
            FlightMode::HeadFree,
            FlightMode::Passthru,
            FlightMode::RangeFinder,
            FlightMode::Failsafe,
        ]
        .into_iter()
        .filter(|&mode| self.is_mode_set(mode))
        .collect()
    }
}
