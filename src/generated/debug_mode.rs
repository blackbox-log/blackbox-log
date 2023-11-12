#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// The configured debugging info for a log.
#[non_exhaustive]
pub enum DebugMode {
    /// `AC_CORRECTION`
    AcCorrection,
    /// `AC_ERROR`
    AcError,
    /// `ACC`
    Acc,
    /// `ACCELEROMETER`
    Accelerometer,
    /// `ACRO_TRAINER`
    AcroTrainer,
    /// `ADC_INTERNAL`
    AdcInternal,
    /// `AGL`
    Agl,
    /// `ALTITUDE`
    Altitude,
    /// `ALWAYS`
    Always,
    /// `ANGLE_MODE`
    AngleMode,
    /// `ANGLE_TARGET`
    AngleTarget,
    /// `ANGLERATE`
    Anglerate,
    /// `ANTI_GRAVITY`
    AntiGravity,
    /// `ATTITUDE`
    Attitude,
    /// `AUTOLEVEL`
    AutoLevel,
    /// `AUTOTRIM`
    AutoTrim,
    /// `AUTOTUNE`
    AutoTune,
    /// `BARO`
    Baro,
    /// `BATTERY`
    Battery,
    /// `BLACKBOX_OUTPUT`
    BlackboxOutput,
    /// `CRSF_LINK_STATISTICS_DOWN`
    CrsfLinkStatisticsDown,
    /// `CRSF_LINK_STATISTICS_PWR`
    CrsfLinkStatisticsPwr,
    /// `CRSF_LINK_STATISTICS_UPLINK`
    CrsfLinkStatisticsUplink,
    /// `CRUISE`
    Cruise,
    /// `CURRENT_ANGLE`
    CurrentAngle,
    /// `CURRENT_SENSOR`
    CurrentSensor,
    /// `CYCLETIME`
    Cycletime,
    /// `D_LPF`
    DLpf,
    /// `D_MIN`
    DMin,
    /// `DSHOT_RPM_ERRORS`
    DshotRpmErrors,
    /// `DSHOT_RPM_TELEMETRY`
    DshotRpmTelemetry,
    /// `DSHOT_TELEMETRY_COUNTS`
    DshotTelemetryCounts,
    /// `DUAL_GYRO_DIFF`
    DualGyroDiff,
    /// `DUAL_GYRO_RAW`
    DualGyroRaw,
    /// `DUAL_GYRO_SCALED`
    DualGyroScaled,
    /// `DYN_IDLE`
    DynIdle,
    /// `DYN_LPF`
    DynLpf,
    /// `DYNAMIC_FILTER`
    DynamicFilter,
    /// `DYNAMIC_FILTER_FREQUENCY`
    DynamicFilterFrequency,
    /// `DYNAMIC_GYRO_LPF`
    DynamicGyroLpf,
    /// `ERPM`
    Erpm,
    /// `ESC_SENSOR`
    EscSensor,
    /// `ESC_SENSOR_RPM`
    EscSensorRpm,
    /// `ESC_SENSOR_TMP`
    EscSensorTmp,
    /// `EZLANDING`
    Ezlanding,
    /// `FAILSAFE`
    Failsafe,
    /// `FEEDFORWARD`
    Feedforward,
    /// `FEEDFORWARD_LIMIT`
    FeedforwardLimit,
    /// `FF_INTERPOLATED`
    FfInterpolated,
    /// `FF_LIMIT`
    FfLimit,
    /// `FFT`
    Fft,
    /// `FFT_FREQ`
    FftFreq,
    /// `FFT_TIME`
    FftTime,
    /// `FLOW`
    Flow,
    /// `FLOW_RAW`
    FlowRaw,
    /// `FPORT`
    Fport,
    /// `GHST`
    Ghst,
    /// `GHST_MSP`
    GhstMsp,
    /// `GPS_CONNECTION`
    GpsConnection,
    /// `GPS_DOP`
    GpsDop,
    /// `GPS_RESCUE_HEADING`
    GpsRescueHeading,
    /// `GPS_RESCUE_THROTTLE_PID`
    GpsRescueThrottlePid,
    /// `GPS_RESCUE_TRACKING`
    GpsRescueTracking,
    /// `GPS_RESCUE_VELOCITY`
    GpsRescueVelocity,
    /// `GYRO`
    Gyro,
    /// `GYRO_CALIBRATION`
    GyroCalibration,
    /// `GYRO_FILTERED`
    GyroFiltered,
    /// `GYRO_RAW`
    GyroRaw,
    /// `GYRO_SAMPLE`
    GyroSample,
    /// `GYRO_SCALED`
    GyroScaled,
    /// `IMU2`
    Imu2,
    /// `IRLOCK`
    Irlock,
    /// `ITERM_RELAX`
    ItermRelax,
    /// `KALMAN_GAIN`
    KalmanGain,
    /// `LANDING`
    Landing,
    /// `LIDAR_TF`
    LidarTf,
    /// `MAG_CALIB`
    MagCalib,
    /// `MAG_TASK_RATE`
    MagTaskRate,
    /// `MAX7456_SIGNAL`
    Max7456Signal,
    /// `MAX7456_SPICLOCK`
    Max7456Spiclock,
    /// `NAV_YAW`
    NavYaw,
    /// `NONE`
    None,
    /// `PCF8574`
    Pcf8574,
    /// `PID_MEASUREMENT`
    PidMeasurement,
    /// `PIDLOOP`
    Pidloop,
    /// `POS_EST`
    PosEst,
    /// `RANGEFINDER`
    Rangefinder,
    /// `RANGEFINDER_QUALITY`
    RangefinderQuality,
    /// `RATE_DYNAMICS`
    RateDynamics,
    /// `RC_INTERPOLATION`
    RcInterpolation,
    /// `RC_SMOOTHING`
    RcSmoothing,
    /// `RC_SMOOTHING_RATE`
    RcSmoothingRate,
    /// `RC_STATS`
    RcStats,
    /// `REM_FLIGHT_TIME`
    RemFlightTime,
    /// `RPM_FILTER`
    RpmFilter,
    /// `RPM_FREQ`
    RpmFreq,
    /// `RPM_LIMIT`
    RpmLimit,
    /// `RTH`
    Rth,
    /// `RUNAWAY_TAKEOFF`
    RunawayTakeoff,
    /// `RX_EXPRESSLRS_PHASELOCK`
    RxExpresslrsPhaselock,
    /// `RX_EXPRESSLRS_SPI`
    RxExpresslrsSpi,
    /// `RX_FRSKY_SPI`
    RxFrskySpi,
    /// `RX_SFHSS_SPI`
    RxSfhssSpi,
    /// `RX_SIGNAL_LOSS`
    RxSignalLoss,
    /// `RX_SPEKTRUM_SPI`
    RxSpektrumSpi,
    /// `RX_STATE_TIME`
    RxStateTime,
    /// `RX_TIMING`
    RxTiming,
    /// `SAG_COMP_VOLTAGE`
    SagCompVoltage,
    /// `SBUS`
    Sbus,
    /// `SCHEDULER`
    Scheduler,
    /// `SCHEDULER_DETERMINISM`
    SchedulerDeterminism,
    /// `SDIO`
    Sdio,
    /// `SMARTAUDIO`
    Smartaudio,
    /// `SMITH_PREDICTOR`
    SmithPredictor,
    /// `SPM_CELLS`
    SpmCells,
    /// `SPM_VARIO`
    SpmVario,
    /// `SPM_VS600`
    SpmVs600,
    /// `STACK`
    Stack,
    /// `TIMING_ACCURACY`
    TimingAccuracy,
    /// `USB`
    Usb,
    /// `VIBE`
    Vibe,
    /// `VTX_MSP`
    VtxMsp,
    /// `VTX_TRAMP`
    VtxTramp,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for DebugMode {
    fn as_name(&self) -> &'static str {
        match self {
            Self::AcCorrection => "AC_CORRECTION",
            Self::AcError => "AC_ERROR",
            Self::Acc => "ACC",
            Self::Accelerometer => "ACCELEROMETER",
            Self::AcroTrainer => "ACRO_TRAINER",
            Self::AdcInternal => "ADC_INTERNAL",
            Self::Agl => "AGL",
            Self::Altitude => "ALTITUDE",
            Self::Always => "ALWAYS",
            Self::AngleMode => "ANGLE_MODE",
            Self::AngleTarget => "ANGLE_TARGET",
            Self::Anglerate => "ANGLERATE",
            Self::AntiGravity => "ANTI_GRAVITY",
            Self::Attitude => "ATTITUDE",
            Self::AutoLevel => "AUTOLEVEL",
            Self::AutoTrim => "AUTOTRIM",
            Self::AutoTune => "AUTOTUNE",
            Self::Baro => "BARO",
            Self::Battery => "BATTERY",
            Self::BlackboxOutput => "BLACKBOX_OUTPUT",
            Self::CrsfLinkStatisticsDown => "CRSF_LINK_STATISTICS_DOWN",
            Self::CrsfLinkStatisticsPwr => "CRSF_LINK_STATISTICS_PWR",
            Self::CrsfLinkStatisticsUplink => "CRSF_LINK_STATISTICS_UPLINK",
            Self::Cruise => "CRUISE",
            Self::CurrentAngle => "CURRENT_ANGLE",
            Self::CurrentSensor => "CURRENT_SENSOR",
            Self::Cycletime => "CYCLETIME",
            Self::DLpf => "D_LPF",
            Self::DMin => "D_MIN",
            Self::DshotRpmErrors => "DSHOT_RPM_ERRORS",
            Self::DshotRpmTelemetry => "DSHOT_RPM_TELEMETRY",
            Self::DshotTelemetryCounts => "DSHOT_TELEMETRY_COUNTS",
            Self::DualGyroDiff => "DUAL_GYRO_DIFF",
            Self::DualGyroRaw => "DUAL_GYRO_RAW",
            Self::DualGyroScaled => "DUAL_GYRO_SCALED",
            Self::DynIdle => "DYN_IDLE",
            Self::DynLpf => "DYN_LPF",
            Self::DynamicFilter => "DYNAMIC_FILTER",
            Self::DynamicFilterFrequency => "DYNAMIC_FILTER_FREQUENCY",
            Self::DynamicGyroLpf => "DYNAMIC_GYRO_LPF",
            Self::Erpm => "ERPM",
            Self::EscSensor => "ESC_SENSOR",
            Self::EscSensorRpm => "ESC_SENSOR_RPM",
            Self::EscSensorTmp => "ESC_SENSOR_TMP",
            Self::Ezlanding => "EZLANDING",
            Self::Failsafe => "FAILSAFE",
            Self::Feedforward => "FEEDFORWARD",
            Self::FeedforwardLimit => "FEEDFORWARD_LIMIT",
            Self::FfInterpolated => "FF_INTERPOLATED",
            Self::FfLimit => "FF_LIMIT",
            Self::Fft => "FFT",
            Self::FftFreq => "FFT_FREQ",
            Self::FftTime => "FFT_TIME",
            Self::Flow => "FLOW",
            Self::FlowRaw => "FLOW_RAW",
            Self::Fport => "FPORT",
            Self::Ghst => "GHST",
            Self::GhstMsp => "GHST_MSP",
            Self::GpsConnection => "GPS_CONNECTION",
            Self::GpsDop => "GPS_DOP",
            Self::GpsRescueHeading => "GPS_RESCUE_HEADING",
            Self::GpsRescueThrottlePid => "GPS_RESCUE_THROTTLE_PID",
            Self::GpsRescueTracking => "GPS_RESCUE_TRACKING",
            Self::GpsRescueVelocity => "GPS_RESCUE_VELOCITY",
            Self::Gyro => "GYRO",
            Self::GyroCalibration => "GYRO_CALIBRATION",
            Self::GyroFiltered => "GYRO_FILTERED",
            Self::GyroRaw => "GYRO_RAW",
            Self::GyroSample => "GYRO_SAMPLE",
            Self::GyroScaled => "GYRO_SCALED",
            Self::Imu2 => "IMU2",
            Self::Irlock => "IRLOCK",
            Self::ItermRelax => "ITERM_RELAX",
            Self::KalmanGain => "KALMAN_GAIN",
            Self::Landing => "LANDING",
            Self::LidarTf => "LIDAR_TF",
            Self::MagCalib => "MAG_CALIB",
            Self::MagTaskRate => "MAG_TASK_RATE",
            Self::Max7456Signal => "MAX7456_SIGNAL",
            Self::Max7456Spiclock => "MAX7456_SPICLOCK",
            Self::NavYaw => "NAV_YAW",
            Self::None => "NONE",
            Self::Pcf8574 => "PCF8574",
            Self::PidMeasurement => "PID_MEASUREMENT",
            Self::Pidloop => "PIDLOOP",
            Self::PosEst => "POS_EST",
            Self::Rangefinder => "RANGEFINDER",
            Self::RangefinderQuality => "RANGEFINDER_QUALITY",
            Self::RateDynamics => "RATE_DYNAMICS",
            Self::RcInterpolation => "RC_INTERPOLATION",
            Self::RcSmoothing => "RC_SMOOTHING",
            Self::RcSmoothingRate => "RC_SMOOTHING_RATE",
            Self::RcStats => "RC_STATS",
            Self::RemFlightTime => "REM_FLIGHT_TIME",
            Self::RpmFilter => "RPM_FILTER",
            Self::RpmFreq => "RPM_FREQ",
            Self::RpmLimit => "RPM_LIMIT",
            Self::Rth => "RTH",
            Self::RunawayTakeoff => "RUNAWAY_TAKEOFF",
            Self::RxExpresslrsPhaselock => "RX_EXPRESSLRS_PHASELOCK",
            Self::RxExpresslrsSpi => "RX_EXPRESSLRS_SPI",
            Self::RxFrskySpi => "RX_FRSKY_SPI",
            Self::RxSfhssSpi => "RX_SFHSS_SPI",
            Self::RxSignalLoss => "RX_SIGNAL_LOSS",
            Self::RxSpektrumSpi => "RX_SPEKTRUM_SPI",
            Self::RxStateTime => "RX_STATE_TIME",
            Self::RxTiming => "RX_TIMING",
            Self::SagCompVoltage => "SAG_COMP_VOLTAGE",
            Self::Sbus => "SBUS",
            Self::Scheduler => "SCHEDULER",
            Self::SchedulerDeterminism => "SCHEDULER_DETERMINISM",
            Self::Sdio => "SDIO",
            Self::Smartaudio => "SMARTAUDIO",
            Self::SmithPredictor => "SMITH_PREDICTOR",
            Self::SpmCells => "SPM_CELLS",
            Self::SpmVario => "SPM_VARIO",
            Self::SpmVs600 => "SPM_VS600",
            Self::Stack => "STACK",
            Self::TimingAccuracy => "TIMING_ACCURACY",
            Self::Usb => "USB",
            Self::Vibe => "VIBE",
            Self::VtxMsp => "VTX_MSP",
            Self::VtxTramp => "VTX_TRAMP",
        }
    }
}
#[allow(unused_qualifications)]
impl ::core::fmt::Display for DebugMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let s = <Self as crate::units::Flag>::as_name(self);
        f.write_str(s)
    }
}
#[allow(
    unused_qualifications,
    clippy::enum_glob_use,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl DebugMode {
    pub(crate) fn new(raw: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        use crate::headers::InternalFirmware::*;
        match (raw, fw) {
            (
                0u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7,
            ) => Some(Self::None),
            (1u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Cycletime)
            }
            (1u32, Inav5) => Some(Self::Gyro),
            (1u32, Inav6 | Inav7) => Some(Self::Agl),
            (2u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Battery)
            }
            (2u32, Inav5) => Some(Self::Agl),
            (2u32, Inav6 | Inav7) => Some(Self::FlowRaw),
            (3u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::GyroFiltered)
            }
            (3u32, Inav5) => Some(Self::FlowRaw),
            (3u32, Inav6 | Inav7) => Some(Self::Flow),
            (4u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Accelerometer)
            }
            (4u32, Inav5) => Some(Self::Flow),
            (4u32, Inav6 | Inav7) => Some(Self::Always),
            (5u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Pidloop)
            }
            (5u32, Inav5) => Some(Self::Sbus),
            (5u32, Inav6 | Inav7) => Some(Self::SagCompVoltage),
            (6u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::GyroScaled)
            }
            (6u32, Inav5) => Some(Self::Fport),
            (6u32, Inav6 | Inav7) => Some(Self::Vibe),
            (7u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RcInterpolation)
            }
            (7u32, Inav5) => Some(Self::Always),
            (7u32, Inav6 | Inav7) => Some(Self::Cruise),
            (8u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Anglerate)
            }
            (8u32, Inav5) => Some(Self::SagCompVoltage),
            (8u32, Inav6 | Inav7) => Some(Self::RemFlightTime),
            (9u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::EscSensor)
            }
            (9u32, Inav5) => Some(Self::Vibe),
            (9u32, Inav6 | Inav7) => Some(Self::Smartaudio),
            (10u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Scheduler)
            }
            (10u32, Inav5) => Some(Self::Cruise),
            (10u32, Inav6 | Inav7) => Some(Self::Acc),
            (11u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Stack)
            }
            (11u32, Inav5) => Some(Self::RemFlightTime),
            (11u32, Inav6 | Inav7) => Some(Self::NavYaw),
            (12u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::EscSensorRpm)
            }
            (12u32, Inav5) => Some(Self::Smartaudio),
            (12u32, Inav6 | Inav7) => Some(Self::Pcf8574),
            (13u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::EscSensorTmp)
            }
            (13u32, Inav5) => Some(Self::Acc),
            (13u32, Inav6 | Inav7) => Some(Self::DynamicGyroLpf),
            (14u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Altitude)
            }
            (14u32, Inav5) => Some(Self::Erpm),
            (14u32, Inav6 | Inav7) => Some(Self::AutoLevel),
            (15u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Fft)
            }
            (15u32, Inav5) => Some(Self::RpmFilter),
            (15u32, Inav6 | Inav7) => Some(Self::Altitude),
            (16u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::FftTime)
            }
            (16u32, Inav5) => Some(Self::RpmFreq),
            (16u32, Inav6 | Inav7) => Some(Self::AutoTrim),
            (17u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::FftFreq)
            }
            (17u32, Inav5) => Some(Self::NavYaw),
            (17u32, Inav6 | Inav7) => Some(Self::AutoTune),
            (18u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RxFrskySpi)
            }
            (18u32, Inav5) => Some(Self::DynamicFilter),
            (18u32, Inav6 | Inav7) => Some(Self::RateDynamics),
            (19u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RxSfhssSpi)
            }
            (19u32, Inav5) => Some(Self::DynamicFilterFrequency),
            (19u32, Inav6 | Inav7) => Some(Self::Landing),
            (20u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::GyroRaw)
            }
            (20u32, Inav5) => Some(Self::Irlock),
            (20u32, Inav6 | Inav7) => Some(Self::PosEst),
            (21u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DualGyroRaw)
            }
            (21u32, Inav5) => Some(Self::KalmanGain),
            (22u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DualGyroDiff)
            }
            (22u32, Inav5) => Some(Self::PidMeasurement),
            (23u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Max7456Signal)
            }
            (23u32, Inav5) => Some(Self::SpmCells),
            (24u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Max7456Spiclock)
            }
            (24u32, Inav5) => Some(Self::SpmVs600),
            (25u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Sbus)
            }
            (25u32, Inav5) => Some(Self::SpmVario),
            (26u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Fport)
            }
            (26u32, Inav5) => Some(Self::Pcf8574),
            (27u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Rangefinder)
            }
            (27u32, Inav5) => Some(Self::DynamicGyroLpf),
            (28u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RangefinderQuality)
            }
            (28u32, Inav5) => Some(Self::AutoLevel),
            (29u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::LidarTf)
            }
            (29u32, Inav5) => Some(Self::Imu2),
            (30u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AdcInternal)
            }
            (30u32, Inav5) => Some(Self::Altitude),
            (31u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RunawayTakeoff)
            }
            (31u32, Inav5) => Some(Self::SmithPredictor),
            (32u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Sdio)
            }
            (32u32, Inav5) => Some(Self::AutoTrim),
            (33u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::CurrentSensor)
            }
            (33u32, Inav5) => Some(Self::AutoTune),
            (34u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Usb)
            }
            (34u32, Inav5) => Some(Self::RateDynamics),
            (35u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Smartaudio)
            }
            (35u32, Inav5) => Some(Self::Landing),
            (36u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Rth)
            }
            (37u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::ItermRelax)
            }
            (38u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AcroTrainer)
            }
            (39u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RcSmoothing)
            }
            (40u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RxSignalLoss)
            }
            (41u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RcSmoothingRate)
            }
            (42u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AntiGravity)
            }
            (43u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DynLpf)
            }
            (44u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RxSpektrumSpi)
            }
            (45u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DshotRpmTelemetry)
            }
            (46u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RpmFilter)
            }
            (47u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DMin)
            }
            (48u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AcCorrection)
            }
            (49u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AcError)
            }
            (50u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DualGyroScaled)
            }
            (51u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DshotRpmErrors)
            }
            (52u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::CrsfLinkStatisticsUplink)
            }
            (53u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::CrsfLinkStatisticsPwr)
            }
            (54u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::CrsfLinkStatisticsDown)
            }
            (55u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Baro)
            }
            (56u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::GpsRescueThrottlePid)
            }
            (57u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::DynIdle)
            }
            (58u32, Betaflight4_2) => Some(Self::FfLimit),
            (58u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::FeedforwardLimit),
            (59u32, Betaflight4_2) => Some(Self::FfInterpolated),
            (59u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Feedforward),
            (60u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::BlackboxOutput)
            }
            (61u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::GyroSample)
            }
            (62u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::RxTiming)
            }
            (63u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::DLpf),
            (64u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::VtxTramp),
            (65u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Ghst),
            (66u32, Betaflight4_3) => Some(Self::SchedulerDeterminism),
            (66u32, Betaflight4_4 | Betaflight4_5) => Some(Self::GhstMsp),
            (67u32, Betaflight4_3) => Some(Self::TimingAccuracy),
            (67u32, Betaflight4_4 | Betaflight4_5) => Some(Self::SchedulerDeterminism),
            (68u32, Betaflight4_3) => Some(Self::RxExpresslrsSpi),
            (68u32, Betaflight4_4 | Betaflight4_5) => Some(Self::TimingAccuracy),
            (69u32, Betaflight4_3) => Some(Self::RxExpresslrsPhaselock),
            (69u32, Betaflight4_4 | Betaflight4_5) => Some(Self::RxExpresslrsSpi),
            (70u32, Betaflight4_3) => Some(Self::RxStateTime),
            (70u32, Betaflight4_4 | Betaflight4_5) => Some(Self::RxExpresslrsPhaselock),
            (71u32, Betaflight4_4 | Betaflight4_5) => Some(Self::RxStateTime),
            (72u32, Betaflight4_4 | Betaflight4_5) => Some(Self::GpsRescueVelocity),
            (73u32, Betaflight4_4 | Betaflight4_5) => Some(Self::GpsRescueHeading),
            (74u32, Betaflight4_4 | Betaflight4_5) => Some(Self::GpsRescueTracking),
            (75u32, Betaflight4_4) => Some(Self::Attitude),
            (75u32, Betaflight4_5) => Some(Self::GpsConnection),
            (76u32, Betaflight4_4) => Some(Self::VtxMsp),
            (76u32, Betaflight4_5) => Some(Self::Attitude),
            (77u32, Betaflight4_4) => Some(Self::GpsDop),
            (77u32, Betaflight4_5) => Some(Self::VtxMsp),
            (78u32, Betaflight4_4) => Some(Self::Failsafe),
            (78u32, Betaflight4_5) => Some(Self::GpsDop),
            (79u32, Betaflight4_5) => Some(Self::Failsafe),
            (80u32, Betaflight4_5) => Some(Self::GyroCalibration),
            (81u32, Betaflight4_5) => Some(Self::AngleMode),
            (82u32, Betaflight4_5) => Some(Self::AngleTarget),
            (83u32, Betaflight4_5) => Some(Self::CurrentAngle),
            (84u32, Betaflight4_5) => Some(Self::DshotTelemetryCounts),
            (85u32, Betaflight4_5) => Some(Self::RpmLimit),
            (86u32, Betaflight4_5) => Some(Self::RcStats),
            (87u32, Betaflight4_5) => Some(Self::MagCalib),
            (88u32, Betaflight4_5) => Some(Self::MagTaskRate),
            (89u32, Betaflight4_5) => Some(Self::Ezlanding),
            _ => {
                #[allow(clippy::redundant_closure_call)]
                (|raw| tracing::error!("invalid debug mode: {raw}"))(raw);
                None
            }
        }
    }
}
