use crate::*;

const LEGACY_ANIFILE_COUNT: usize = 0x100;
const LEGACY_ANIMATION_COUNT: usize = 0x400;
const LEGACY_SPRITEFRAME_COUNT: usize = 0x1000;

const LEGACY_HITBOX_COUNT: usize = 0x20;
const LEGACY_HITBOX_DIR_COUNT: usize = 0x8;

pub enum AnimRotationFlags {
    ROTSTYLE_NONE,
    ROTSTYLE_FULL,
    ROTSTYLE_45DEG,
    ROTSTYLE_STATICFRAMES,
}

#[repr(C)]
pub struct AnimationFile {
    fileName: [i8; 0x20],
    animCount: int32,
    aniListOffset: int32,
    hitboxListOffset: int32,
}
const DEFAULT_ANIMATIONFILE: AnimationFile = AnimationFile {
    fileName: [0; 0x20],
    animCount: 0,
    aniListOffset: 0,
    hitboxListOffset: 0,
};

#[repr(C)]
pub struct SpriteAnimation {
    name: [i8; 16],
    frameCount: uint8,
    speed: uint8,
    loopPoint: uint8,
    rotationStyle: uint8,
    frameListOffset: int32,
}
const DEFAULT_SPRITEANIMATION: SpriteAnimation = SpriteAnimation {
    name: [0; 16],
    frameCount: 0,
    speed: 0,
    loopPoint: 0,
    rotationStyle: 0,
    frameListOffset: 0,
};

#[repr(C)]
pub struct SpriteFrame {
    sprX: int32,
    sprY: int32,
    width: int32,
    height: int32,
    pivotX: int32,
    pivotY: int32,
    sheetID: uint8,
    hitboxID: uint8,
}
const DEFAULT_SPRITEFRAME: SpriteFrame = SpriteFrame {
    sprX: 0,
    sprY: 0,
    width: 0,
    height: 0,
    pivotX: 0,
    pivotY: 0,
    sheetID: 0,
    hitboxID: 0,
};

#[repr(C)]
struct Hitbox {
    left: [int8; LEGACY_HITBOX_DIR_COUNT],
    top: [int8; LEGACY_HITBOX_DIR_COUNT],
    right: [int8; LEGACY_HITBOX_DIR_COUNT],
    bottom: [int8; LEGACY_HITBOX_DIR_COUNT],
}
const DEFAULT_HITBOX: Hitbox = Hitbox {
    left: [0; LEGACY_HITBOX_DIR_COUNT],
    top: [0; LEGACY_HITBOX_DIR_COUNT],
    right: [0; LEGACY_HITBOX_DIR_COUNT],
    bottom: [0; LEGACY_HITBOX_DIR_COUNT],
};

#[no_mangle]
static mut Legacy_animationFileList: [AnimationFile; LEGACY_ANIFILE_COUNT] =
    [DEFAULT_ANIMATIONFILE; LEGACY_ANIFILE_COUNT];
#[no_mangle]
static mut Legacy_animationFileCount: int32 = 0;

#[no_mangle]
static mut Legacy_scriptFrames: [SpriteFrame; LEGACY_SPRITEFRAME_COUNT] =
    [DEFAULT_SPRITEFRAME; LEGACY_SPRITEFRAME_COUNT];
#[no_mangle]
static mut Legacy_scriptFrameCount: int32 = 0;

#[no_mangle]
static mut Legacy_animFrames: [SpriteFrame; LEGACY_SPRITEFRAME_COUNT] =
    [DEFAULT_SPRITEFRAME; LEGACY_SPRITEFRAME_COUNT];
#[no_mangle]
static mut Legacy_animFrameCount: int32 = 0;
#[no_mangle]
static mut Legacy_animationList: [SpriteAnimation; LEGACY_ANIMATION_COUNT] =
    [DEFAULT_SPRITEANIMATION; LEGACY_ANIMATION_COUNT];
#[no_mangle]
static mut Legacy_animationCount: int32 = 0;
#[no_mangle]
static mut Legacy_hitboxList: [Hitbox; LEGACY_HITBOX_COUNT] = [DEFAULT_HITBOX; LEGACY_HITBOX_COUNT];
#[no_mangle]
static mut Legacy_hitboxCount: int32 = 0;
