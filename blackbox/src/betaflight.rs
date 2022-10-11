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

// flightModeFlags_e in runtime_config.h
generate_flight_mode! {
    Angle / angle = 0,
    Horizon / horizon = 1,
    Mag / mag = 2,
    Baro / baro = 3,
    GpsHome / gps_home = 4,
    GpsHold / gps_hold = 5,
    HeadFree / head_free = 6,
    Passthru / passthru = 8,
    RangeFinder / range_finder = 9,
    Failsafe / failsafe = 10,
    GpsRescue / gps_rescue = 11,
}
