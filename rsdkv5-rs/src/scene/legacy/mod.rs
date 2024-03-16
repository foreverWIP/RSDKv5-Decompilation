use crate::*;

pub mod object;
pub mod v3;
pub mod v4;

const LEGACY_SCRIPTDATA_COUNT: usize = 0x40000;
const LEGACY_JUMPTABLE_COUNT: usize = 0x4000;
const LEGACY_FUNCTION_COUNT: usize = 0x200;

const LEGACY_JUMPSTACK_COUNT: usize = 0x400;
const LEGACY_FUNCSTACK_COUNT: usize = 0x400;
const LEGACY_FORSTACK_COUNT: usize = 0x400;

#[repr(C)]
pub struct ScriptPtr {
    scriptCodePtr: int32,
    jumpTablePtr: int32,
}

#[repr(i32)]
pub enum LegacyStageModes {
    STAGEMODE_LOAD,
    STAGEMODE_NORMAL,
    STAGEMODE_PAUSED,

    STAGEMODE_STEPOVER = 8,
}

#[no_mangle]
pub static mut Legacy_stageMode: LegacyStageModes = LegacyStageModes::STAGEMODE_LOAD;
