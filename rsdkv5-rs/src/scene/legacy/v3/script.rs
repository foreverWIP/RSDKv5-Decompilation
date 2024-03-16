use crate::*;

use self::{graphics::legacy::animation::AnimationFile, scene::legacy::ScriptPtr};

#[repr(C)]
pub struct ScriptFunction {
    name: [i8; 0x20],
    ptr: ScriptPtr,
}

#[repr(C)]
pub struct ObjectScript {
    pub frameCount: int32,
    pub spriteSheetID: int32,
    pub subMain: ScriptPtr,
    pub subPlayerInteraction: ScriptPtr,
    pub subDraw: ScriptPtr,
    pub subStartup: ScriptPtr,
    pub frameListOffset: int32,
    pub animFile: *const AnimationFile,
}

#[repr(C)]
pub struct ScriptEngine {
    operands: [int32; 10],
    tempValue: [int32; 8],
    arrayPosition: [int32; 3],
    checkResult: int32,
}

#[repr(C)]
pub enum ScriptSubs {
    SUB_MAIN = 0,
    SUB_PLAYERINTERACTION = 1,
    SUB_DRAW = 2,
    SUB_SETUP = 3,
}
