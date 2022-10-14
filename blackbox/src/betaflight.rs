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

generate_flight_mode! {
    Arm / arm = 0,
    Angle / angle = 1,
    Horizon / horizon = 2,
    Mag / mag = 3,
    HeadFree / head_free = 4,
    Passthru / passthru = 5,
    Failsafe / failsafe = 6,
    GpsRescue / gps_rescue = 7,
    Antigravity / antigravity = 8,
    HeadAdjust / head_adjust = 9,
    CamStab / cam_stab = 10,
    BeeperOn / beeper_on = 11,
    LedLow / led_low = 12,
    Calib / calib = 13,
    Osd / osd = 14,
    Telemetry / telemetry = 15,
    Servo1 / servo1 = 16,
    Servo2 / servo2 = 17,
    Servo3 / servo3 = 18,
    Blackbox / blackbox = 19,
    Airmode / airmode = 20,
    ThreeD / three_d = 21,
    FpvAngleMix / fpv_angle_mix = 22,
    BlackboxErase / blackbox_erase = 23,
    Camera1 / camera1 = 24,
    Camera2 / camera2 = 25,
    Camera3 / camera3 = 26,
    FlipOverAfterCrash / flip_over_after_crash = 27,
    Prearm / prearm = 28,
    BeepGpsCount / beep_gps_count = 29,
    VtxPitmode / vtx_pitmode = 30,
    Paralyze / paralyze = 31,
    // User1 / user1 = 32,
    // User2 / user2 = 33,
    // User3 / user3 = 34,
    // User4 / user4 = 35,
    // PidAudio / pid_audio = 36,
    // AcroTrainer / acro_trainer = 37,
    // VtxControlDisable / vtx_control_disable = 38,
    // LaunchControl / launch_control = 39,
    // MspOverride / msp_override = 40,
    // StickCommandDisable / stick_command_disable = 41,
    // BeeperMute / beeper_mute = 42,
}