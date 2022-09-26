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
