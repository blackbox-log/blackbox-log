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
