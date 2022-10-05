// disarmReason_t in fc_core.h
generate_disarm_reason! {
    None = 0,
    Timeout = 1,
    Sticks = 2,
    Switch3d = 3,
    Switch = 4,
    Killswitch = 5,
    Failsafe = 6,
    Navigation = 7,
    Landing = 8,
}

// flightModeFlags_e in runtime_config.h
generate_flight_mode! {
    Angle / angle = 0,
    Horizon / horizon = 1,
    Heading / heading = 2,
    NavAltHold / nav_alt_hold = 3,
    NavRth / nav_rth = 4,
    NavPoshold / nav_poshold = 5,
    HeadFree / head_free = 6,
    NavLaunch / nav_launch = 7,
    Manual / manual = 8,
    Failsafe / failsafe = 9,
    AutoTune / auto_tune = 10,
    NavWp / nav_wp = 11,
    NavCourseHold / nav_course_hold = 12,
    Flaperon / flaperon = 13,
    TurnAssistant / turn_assistant = 14,
    Turtle / turtle = 15,
    Soaring / soaring = 16,
}
