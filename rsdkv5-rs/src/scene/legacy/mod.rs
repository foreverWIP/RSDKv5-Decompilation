use crate::*;

#[repr(i32)]
pub enum LegacyStageModes {
    STAGEMODE_LOAD,
    STAGEMODE_NORMAL,
    STAGEMODE_PAUSED,

    STAGEMODE_STEPOVER = 8,
}

#[no_mangle]
pub static mut Legacy_stageMode: LegacyStageModes = LegacyStageModes::STAGEMODE_LOAD;
