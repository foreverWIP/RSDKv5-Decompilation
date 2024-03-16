use crate::*;

use self::{graphics::legacy::animation::AnimationFile, scene::legacy::ScriptPtr};

#[repr(C)]
pub struct ObjectScript {
    pub frameCount: int32,
    pub spriteSheetID: int32,
    pub eventUpdate: ScriptPtr,
    pub eventDraw: ScriptPtr,
    pub eventStartup: ScriptPtr,
    pub frameListOffset: int32,
    pub animFile: *const AnimationFile,
}
