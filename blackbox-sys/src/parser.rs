#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use std::ffi::CString;
use std::ptr;

#[allow(missing_debug_implementations)]
pub struct FlightLog(ffi::flightLog_t);

impl FlightLog {
    pub fn new() -> Self {
        Self(ffi::flightLog_t {
            dateTime: 0,
            stats: ffi::flightLogStatistics_t {
                totalBytes: 0,
                totalCorruptFrames: 0,
                intentionallyAbsentIterations: 0,
                haveFieldStats: false,
                field: [ffi::flightLogFieldStatistics_t::default(); 128],
                frame: [ffi::flightLogFrameStatistics_t {
                    bytes: 0,
                    validCount: 0,
                    desyncCount: 0,
                    corruptCount: 0,
                    sizeCount: [0; 257],
                }; 256],
            },
            frameDefs: [ffi::flightLogFrameDef_t {
                namesLine: ptr::null_mut(),
                fieldCount: 0,
                fieldName: [ptr::null_mut(); 128],
                fieldSigned: [0; 128],
                fieldWidth: [0; 128],
                predictor: [0; 128],
                encoding: [0; 128],
            }; 256],
            sysConfig: ffi::flightLogSysConfig_t::default(),
            logBegin: [ptr::null(); 1001],
            logCount: 0,
            frameIntervalI: 0,
            frameIntervalPNum: 0,
            frameIntervalPDenom: 0,
            mainFieldIndexes: ffi::mainFieldIndexes_t::default(),
            gpsFieldIndexes: ffi::gpsGFieldIndexes_t::default(),
            gpsHomeFieldIndexes: ffi::gpsHFieldIndexes_t::default(),
            slowFieldIndexes: ffi::slowFieldIndexes_t::default(),
            private: ptr::null_mut(),
        })
    }

    pub fn sys_config_mut(&mut self) -> &mut ffi::flightLogSysConfig_t {
        &mut self.0.sysConfig
    }

    pub fn vbat_to_millivolts(&mut self, vbat: u16) -> u32 {
        let pointer = ptr::addr_of_mut!(self.0);
        unsafe { ffi::flightLogVbatADCToMillivolts(pointer, vbat) }
    }

    pub fn amperage_to_milliamps(&mut self, amps: u16) -> i32 {
        let pointer = ptr::addr_of_mut!(self.0);
        unsafe { ffi::flightLogAmperageADCToMilliamps(pointer, amps) }
    }

    pub fn gyro_to_rad_per_sec(&mut self, gyro: i32) -> f64 {
        let pointer = ptr::addr_of_mut!(self.0);
        unsafe { ffi::flightlogGyroToRadiansPerSecond(pointer, gyro) }
    }

    pub fn accel_to_gs(&mut self, accel: i32) -> f64 {
        let pointer = ptr::addr_of_mut!(self.0);
        unsafe { ffi::flightlogAccelerationRawToGs(pointer, accel) }
    }
}

impl Default for FlightLog {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn flight_mode_to_string(mode: u32) -> CString {
    let buffer = [0; 100];
    let len = buffer.len() as i32;
    let pointer = CString::new(buffer).unwrap().into_raw();

    unsafe {
        ffi::flightlogFlightModeToString(mode, pointer, len);
        CString::from_raw(pointer)
    }
}

#[inline]
pub fn flight_state_to_string(state: u32) -> CString {
    let buffer = [0; 100];
    let len = buffer.len() as i32;
    let pointer = CString::new(buffer).unwrap().into_raw();

    unsafe {
        ffi::flightlogFlightStateToString(state, pointer, len);
        CString::from_raw(pointer)
    }
}

#[inline]
pub fn failsafe_phase_to_string(phase: u8) -> CString {
    let buffer = [0; 100];
    let len = buffer.len() as i32;
    let pointer = CString::new(buffer).unwrap().into_raw();

    unsafe {
        ffi::flightlogFailsafePhaseToString(phase, pointer, len);
        CString::from_raw(pointer)
    }
}

#[allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
mod ffi {
    use std::ffi::{c_char, c_int, c_long, c_uint, c_void};
    #[cfg(test)]
    use std::mem::{self, MaybeUninit};
    #[cfg(test)]
    use std::ptr;

    type FirmwareType = c_uint;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub(super) struct flightLogFrameStatistics_t {
        pub(super) bytes: u32,
        pub(super) validCount: u32,
        pub(super) desyncCount: u32,
        pub(super) corruptCount: u32,
        pub(super) sizeCount: [u32; 257],
    }

    #[test]
    fn bindgen_test_layout_flightLogFrameStatistics_t() {
        const UNINIT: MaybeUninit<flightLogFrameStatistics_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<flightLogFrameStatistics_t>(),
            1044,
            concat!("Size of: ", stringify!(flightLogFrameStatistics_t))
        );
        assert_eq!(
            mem::align_of::<flightLogFrameStatistics_t>(),
            4,
            concat!("Alignment of ", stringify!(flightLogFrameStatistics_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).bytes) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameStatistics_t),
                "::",
                stringify!(bytes)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).validCount) as usize - ptr as usize },
            4,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameStatistics_t),
                "::",
                stringify!(validCount)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).desyncCount) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameStatistics_t),
                "::",
                stringify!(desyncCount)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).corruptCount) as usize - ptr as usize },
            12,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameStatistics_t),
                "::",
                stringify!(corruptCount)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).sizeCount) as usize - ptr as usize },
            16,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameStatistics_t),
                "::",
                stringify!(sizeCount)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub(super) struct flightLogFieldStatistics_t {
        min: i64,
        max: i64,
    }

    #[test]
    fn bindgen_test_layout_flightLogFieldStatistics_t() {
        const UNINIT: MaybeUninit<flightLogFieldStatistics_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<flightLogFieldStatistics_t>(),
            16,
            concat!("Size of: ", stringify!(flightLogFieldStatistics_t))
        );
        assert_eq!(
            mem::align_of::<flightLogFieldStatistics_t>(),
            8,
            concat!("Alignment of ", stringify!(flightLogFieldStatistics_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).min) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFieldStatistics_t),
                "::",
                stringify!(min)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).max) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFieldStatistics_t),
                "::",
                stringify!(max)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub(super) struct flightLogStatistics_t {
        pub(super) totalBytes: u32,
        pub(super) totalCorruptFrames: u32,
        pub(super) intentionallyAbsentIterations: u32,
        pub(super) haveFieldStats: bool,
        pub(super) field: [flightLogFieldStatistics_t; 128],
        pub(super) frame: [flightLogFrameStatistics_t; 256],
    }

    #[test]
    fn bindgen_test_layout_flightLogStatistics_t() {
        const UNINIT: MaybeUninit<flightLogStatistics_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<flightLogStatistics_t>(),
            269328,
            concat!("Size of: ", stringify!(flightLogStatistics_t))
        );
        assert_eq!(
            mem::align_of::<flightLogStatistics_t>(),
            8,
            concat!("Alignment of ", stringify!(flightLogStatistics_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).totalBytes) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(flightLogStatistics_t),
                "::",
                stringify!(totalBytes)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).totalCorruptFrames) as usize - ptr as usize },
            4,
            concat!(
                "Offset of field: ",
                stringify!(flightLogStatistics_t),
                "::",
                stringify!(totalCorruptFrames)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).intentionallyAbsentIterations) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(flightLogStatistics_t),
                "::",
                stringify!(intentionallyAbsentIterations)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).haveFieldStats) as usize - ptr as usize },
            12,
            concat!(
                "Offset of field: ",
                stringify!(flightLogStatistics_t),
                "::",
                stringify!(haveFieldStats)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).field) as usize - ptr as usize },
            16,
            concat!(
                "Offset of field: ",
                stringify!(flightLogStatistics_t),
                "::",
                stringify!(field)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).frame) as usize - ptr as usize },
            2064,
            concat!(
                "Offset of field: ",
                stringify!(flightLogStatistics_t),
                "::",
                stringify!(frame)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub(super) struct gpsGFieldIndexes_t {
        time: ::std::ffi::c_int,
        GPS_numSat: c_int,
        GPS_coord: [c_int; 2],
        GPS_altitude: c_int,
        GPS_speed: c_int,
        GPS_ground_course: c_int,
    }

    #[test]
    fn bindgen_test_layout_gpsGFieldIndexes_t() {
        const UNINIT: MaybeUninit<gpsGFieldIndexes_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<gpsGFieldIndexes_t>(),
            28,
            concat!("Size of: ", stringify!(gpsGFieldIndexes_t))
        );
        assert_eq!(
            mem::align_of::<gpsGFieldIndexes_t>(),
            4,
            concat!("Alignment of ", stringify!(gpsGFieldIndexes_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).time) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(gpsGFieldIndexes_t),
                "::",
                stringify!(time)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).GPS_numSat) as usize - ptr as usize },
            4,
            concat!(
                "Offset of field: ",
                stringify!(gpsGFieldIndexes_t),
                "::",
                stringify!(GPS_numSat)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).GPS_coord) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(gpsGFieldIndexes_t),
                "::",
                stringify!(GPS_coord)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).GPS_altitude) as usize - ptr as usize },
            16,
            concat!(
                "Offset of field: ",
                stringify!(gpsGFieldIndexes_t),
                "::",
                stringify!(GPS_altitude)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).GPS_speed) as usize - ptr as usize },
            20,
            concat!(
                "Offset of field: ",
                stringify!(gpsGFieldIndexes_t),
                "::",
                stringify!(GPS_speed)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).GPS_ground_course) as usize - ptr as usize },
            24,
            concat!(
                "Offset of field: ",
                stringify!(gpsGFieldIndexes_t),
                "::",
                stringify!(GPS_ground_course)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub(super) struct gpsHFieldIndexes_t {
        GPS_home: [c_int; 2],
    }

    #[test]
    fn bindgen_test_layout_gpsHFieldIndexes_t() {
        const UNINIT: MaybeUninit<gpsHFieldIndexes_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<gpsHFieldIndexes_t>(),
            8,
            concat!("Size of: ", stringify!(gpsHFieldIndexes_t))
        );
        assert_eq!(
            mem::align_of::<gpsHFieldIndexes_t>(),
            4,
            concat!("Alignment of ", stringify!(gpsHFieldIndexes_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).GPS_home) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(gpsHFieldIndexes_t),
                "::",
                stringify!(GPS_home)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub(super) struct slowFieldIndexes_t {
        flightModeFlags: c_int,
        stateFlags: c_int,
        failsafePhase: c_int,
    }

    #[test]
    fn bindgen_test_layout_slowFieldIndexes_t() {
        const UNINIT: MaybeUninit<slowFieldIndexes_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<slowFieldIndexes_t>(),
            12,
            concat!("Size of: ", stringify!(slowFieldIndexes_t))
        );
        assert_eq!(
            mem::align_of::<slowFieldIndexes_t>(),
            4,
            concat!("Alignment of ", stringify!(slowFieldIndexes_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).flightModeFlags) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(slowFieldIndexes_t),
                "::",
                stringify!(flightModeFlags)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).stateFlags) as usize - ptr as usize },
            4,
            concat!(
                "Offset of field: ",
                stringify!(slowFieldIndexes_t),
                "::",
                stringify!(stateFlags)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).failsafePhase) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(slowFieldIndexes_t),
                "::",
                stringify!(failsafePhase)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub(super) struct mainFieldIndexes_t {
        loopIteration: c_int,
        time: c_int,
        pid: [[c_int; 3]; 3],
        rcCommand: [c_int; 4],
        vbatLatest: c_int,
        amperageLatest: c_int,
        magADC: [c_int; 3],
        BaroAlt: c_int,
        sonarRaw: c_int,
        rssi: c_int,
        gyroADC: [c_int; 3],
        accSmooth: [c_int; 3],
        motor: [c_int; 8],
        servo: [c_int; 8],
    }

    #[test]
    fn bindgen_test_layout_mainFieldIndexes_t() {
        const UNINIT: MaybeUninit<mainFieldIndexes_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<mainFieldIndexes_t>(),
            180,
            concat!("Size of: ", stringify!(mainFieldIndexes_t))
        );
        assert_eq!(
            mem::align_of::<mainFieldIndexes_t>(),
            4,
            concat!("Alignment of ", stringify!(mainFieldIndexes_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).loopIteration) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(loopIteration)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).time) as usize - ptr as usize },
            4,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(time)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).pid) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(pid)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).rcCommand) as usize - ptr as usize },
            44,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(rcCommand)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).vbatLatest) as usize - ptr as usize },
            60,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(vbatLatest)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).amperageLatest) as usize - ptr as usize },
            64,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(amperageLatest)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).magADC) as usize - ptr as usize },
            68,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(magADC)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).BaroAlt) as usize - ptr as usize },
            80,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(BaroAlt)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).sonarRaw) as usize - ptr as usize },
            84,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(sonarRaw)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).rssi) as usize - ptr as usize },
            88,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(rssi)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).gyroADC) as usize - ptr as usize },
            92,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(gyroADC)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).accSmooth) as usize - ptr as usize },
            104,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(accSmooth)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).motor) as usize - ptr as usize },
            116,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(motor)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).servo) as usize - ptr as usize },
            148,
            concat!(
                "Offset of field: ",
                stringify!(mainFieldIndexes_t),
                "::",
                stringify!(servo)
            )
        );
    }

    /// Information about the system configuration of the craft being logged
    /// (aids in interpretation
    /// of the log data).
    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub struct flightLogSysConfig_t {
        pub(super) minthrottle: c_int,
        pub(super) maxthrottle: c_int,
        pub(super) motorOutputLow: c_int,
        pub(super) motorOutputHigh: c_int,
        pub(super) rcRate: c_uint,
        pub(super) yawRate: c_uint,
        pub acc_1G: u16,
        pub gyroScale: f32,
        pub vbatscale: u8,
        pub(super) vbatmaxcellvoltage: u8,
        pub(super) vbatmincellvoltage: u8,
        pub(super) vbatwarningcellvoltage: u8,
        pub currentMeterOffset: i16,
        pub currentMeterScale: i16,
        pub(super) vbatref: u16,
        pub(super) firmwareType: FirmwareType,
    }

    #[test]
    fn bindgen_test_layout_flightLogSysConfig_t() {
        const UNINIT: MaybeUninit<flightLogSysConfig_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<flightLogSysConfig_t>(),
            48,
            concat!("Size of: ", stringify!(flightLogSysConfig_t))
        );
        assert_eq!(
            mem::align_of::<flightLogSysConfig_t>(),
            4,
            concat!("Alignment of ", stringify!(flightLogSysConfig_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).minthrottle) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(minthrottle)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).maxthrottle) as usize - ptr as usize },
            4,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(maxthrottle)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).motorOutputLow) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(motorOutputLow)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).motorOutputHigh) as usize - ptr as usize },
            12,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(motorOutputHigh)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).rcRate) as usize - ptr as usize },
            16,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(rcRate)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).yawRate) as usize - ptr as usize },
            20,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(yawRate)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).acc_1G) as usize - ptr as usize },
            24,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(acc_1G)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).gyroScale) as usize - ptr as usize },
            28,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(gyroScale)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).vbatscale) as usize - ptr as usize },
            32,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(vbatscale)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).vbatmaxcellvoltage) as usize - ptr as usize },
            33,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(vbatmaxcellvoltage)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).vbatmincellvoltage) as usize - ptr as usize },
            34,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(vbatmincellvoltage)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).vbatwarningcellvoltage) as usize - ptr as usize },
            35,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(vbatwarningcellvoltage)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).currentMeterOffset) as usize - ptr as usize },
            36,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(currentMeterOffset)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).currentMeterScale) as usize - ptr as usize },
            38,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(currentMeterScale)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).vbatref) as usize - ptr as usize },
            40,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(vbatref)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).firmwareType) as usize - ptr as usize },
            44,
            concat!(
                "Offset of field: ",
                stringify!(flightLogSysConfig_t),
                "::",
                stringify!(firmwareType)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub(super) struct flightLogFrameDef_t {
        pub(super) namesLine: *mut c_char,
        pub(super) fieldCount: c_int,
        pub(super) fieldName: [*mut c_char; 128],
        pub(super) fieldSigned: [c_int; 128],
        pub(super) fieldWidth: [c_int; 128],
        pub(super) predictor: [c_int; 128],
        pub(super) encoding: [c_int; 128],
    }

    #[test]
    fn bindgen_test_layout_flightLogFrameDef_t() {
        const UNINIT: MaybeUninit<flightLogFrameDef_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<flightLogFrameDef_t>(),
            3088,
            concat!("Size of: ", stringify!(flightLogFrameDef_t))
        );
        assert_eq!(
            mem::align_of::<flightLogFrameDef_t>(),
            8,
            concat!("Alignment of ", stringify!(flightLogFrameDef_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).namesLine) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(namesLine)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).fieldCount) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(fieldCount)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).fieldName) as usize - ptr as usize },
            16,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(fieldName)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).fieldSigned) as usize - ptr as usize },
            1040,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(fieldSigned)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).fieldWidth) as usize - ptr as usize },
            1552,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(fieldWidth)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).predictor) as usize - ptr as usize },
            2064,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(predictor)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).encoding) as usize - ptr as usize },
            2576,
            concat!(
                "Offset of field: ",
                stringify!(flightLogFrameDef_t),
                "::",
                stringify!(encoding)
            )
        );
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub(super) struct flightLog_t {
        pub(super) dateTime: c_long,
        pub(super) stats: flightLogStatistics_t,
        pub(super) frameDefs: [flightLogFrameDef_t; 256],
        pub(super) sysConfig: flightLogSysConfig_t,
        pub(super) logBegin: [*const c_char; 1001],
        pub(super) logCount: c_int,
        pub(super) frameIntervalI: c_uint,
        pub(super) frameIntervalPNum: c_uint,
        pub(super) frameIntervalPDenom: c_uint,
        pub(super) mainFieldIndexes: mainFieldIndexes_t,
        pub(super) gpsFieldIndexes: gpsGFieldIndexes_t,
        pub(super) gpsHomeFieldIndexes: gpsHFieldIndexes_t,
        pub(super) slowFieldIndexes: slowFieldIndexes_t,
        pub(super) private: *mut c_void,
    }

    #[test]
    fn bindgen_test_layout_flightLog_t() {
        const UNINIT: MaybeUninit<flightLog_t> = MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            mem::size_of::<flightLog_t>(),
            1068176,
            concat!("Size of: ", stringify!(flightLog_t))
        );
        assert_eq!(
            mem::align_of::<flightLog_t>(),
            8,
            concat!("Alignment of ", stringify!(flightLog_t))
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).dateTime) as usize - ptr as usize },
            0,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(dateTime)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).stats) as usize - ptr as usize },
            8,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(stats)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).frameDefs) as usize - ptr as usize },
            269336,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(frameDefs)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).sysConfig) as usize - ptr as usize },
            1059864,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(sysConfig)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).logBegin) as usize - ptr as usize },
            1059912,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(logBegin)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).logCount) as usize - ptr as usize },
            1067920,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(logCount)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).frameIntervalI) as usize - ptr as usize },
            1067924,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(frameIntervalI)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).frameIntervalPNum) as usize - ptr as usize },
            1067928,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(frameIntervalPNum)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).frameIntervalPDenom) as usize - ptr as usize },
            1067932,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(frameIntervalPDenom)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).mainFieldIndexes) as usize - ptr as usize },
            1067936,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(mainFieldIndexes)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).gpsFieldIndexes) as usize - ptr as usize },
            1068116,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(gpsFieldIndexes)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).gpsHomeFieldIndexes) as usize - ptr as usize },
            1068144,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(gpsHomeFieldIndexes)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).slowFieldIndexes) as usize - ptr as usize },
            1068152,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(slowFieldIndexes)
            )
        );
        assert_eq!(
            unsafe { ptr::addr_of!((*ptr).private) as usize - ptr as usize },
            1068168,
            concat!(
                "Offset of field: ",
                stringify!(flightLog_t),
                "::",
                stringify!(private)
            )
        );
    }

    extern "C" {
        // pub(super) fn flightLogEstimateNumCells(log: *mut flightLog_t) -> c_int;
        pub(super) fn flightLogVbatADCToMillivolts(
            log: *mut flightLog_t,
            vbatADC: u16,
        ) -> ::std::ffi::c_uint;
        pub(super) fn flightLogAmperageADCToMilliamps(
            log: *mut flightLog_t,
            amperageADC: u16,
        ) -> ::std::ffi::c_int;
        pub(super) fn flightlogGyroToRadiansPerSecond(log: *mut flightLog_t, gyroRaw: i32) -> f64;
        pub(super) fn flightlogAccelerationRawToGs(log: *mut flightLog_t, accRaw: i32) -> f64;
        pub(super) fn flightlogFlightModeToString(
            flightMode: u32,
            dest: *mut ::std::ffi::c_char,
            destLen: ::std::ffi::c_int,
        );
        pub(super) fn flightlogFlightStateToString(
            flightState: u32,
            dest: *mut ::std::ffi::c_char,
            destLen: ::std::ffi::c_int,
        );
        pub(super) fn flightlogFailsafePhaseToString(
            failsafePhase: u8,
            dest: *mut ::std::ffi::c_char,
            destLen: ::std::ffi::c_int,
        );
    }
}
