use std::ffi::CStr;

use crate::*;

use self::engine_core::reader::{
    close_file, init_file_info, load_file, read_int_8, read_string, FileModes, DEFAULT_FILEINFO,
};

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
    pub fileName: [i8; 0x20],
    pub animCount: int32,
    pub aniListOffset: int32,
    pub hitboxListOffset: int32,
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

extern "C" {
    fn AddGraphicsFile(filePath: *const i8) -> int32;
}

#[no_mangle]
#[export_name = "Legacy_LoadAnimationFile"]
pub extern "C" fn legacy_load_animation_file(filePath: *const i8) {
    let mut info = DEFAULT_FILEINFO;
    init_file_info(&mut info);

    if (load_file(&mut info, filePath, FileModes::FMODE_RB as u8) == true32) {
        unsafe {
            let mut sheetIDs = [0u8; 24];
            sheetIDs[0] = 0;

            let sheetCount: uint8 = read_int_8(&mut info) as u8;

            for s in 0..sheetCount {
                let sheetPath = read_string(&mut info) + "\0";
                sheetIDs[s as usize] = AddGraphicsFile(sheetPath.as_ptr() as *const i8) as u8;
            }

            let animFile = &mut Legacy_animationFileList[Legacy_animationFileCount as usize];
            animFile.animCount = read_int_8(&mut info) as i32;
            animFile.aniListOffset = Legacy_animationCount;

            for a in 0..animFile.animCount {
                let anim = &mut Legacy_animationList[Legacy_animationCount as usize];
                Legacy_animationCount += 1;
                anim.frameListOffset = Legacy_animFrameCount;

                let anim_name_string = read_string(&mut info);
                anim_name_string
                    .as_ptr()
                    .copy_to(anim.name.as_ptr() as *mut u8, anim_name_string.len());
                anim.frameCount = read_int_8(&mut info);
                anim.speed = read_int_8(&mut info);
                anim.loopPoint = read_int_8(&mut info);
                anim.rotationStyle = read_int_8(&mut info);

                for f in 0..anim.frameCount {
                    let frame = &mut Legacy_animFrames[Legacy_animFrameCount as usize];
                    Legacy_animFrameCount += 1;
                    frame.sheetID = sheetIDs[read_int_8(&mut info) as usize];
                    frame.hitboxID = read_int_8(&mut info);
                    frame.sprX = read_int_8(&mut info) as i32;
                    frame.sprY = read_int_8(&mut info) as i32;
                    frame.width = read_int_8(&mut info) as i32;
                    frame.height = read_int_8(&mut info) as i32;
                    frame.pivotX = (read_int_8(&mut info) as i8) as i32;
                    frame.pivotY = (read_int_8(&mut info) as i8) as i32;
                }

                // 90 Degree (Extra rotation Frames) rotation
                if (anim.rotationStyle == AnimRotationFlags::ROTSTYLE_STATICFRAMES as u8) {
                    anim.frameCount >>= 1;
                }
            }

            animFile.hitboxListOffset = Legacy_hitboxCount;
            let hbCount: int32 = read_int_8(&mut info) as i32;
            for h in 0..hbCount {
                let hitbox = &mut Legacy_hitboxList[Legacy_hitboxCount as usize];
                Legacy_hitboxCount += 1;
                for d in 0..LEGACY_HITBOX_DIR_COUNT {
                    hitbox.left[d] = read_int_8(&mut info) as i8;
                    hitbox.top[d] = read_int_8(&mut info) as i8;
                    hitbox.right[d] = read_int_8(&mut info) as i8;
                    hitbox.bottom[d] = read_int_8(&mut info) as i8;
                }
            }
        }

        close_file(&mut info);
    }
}

#[no_mangle]
#[export_name = "Legacy_ClearAnimationData"]
pub extern "C" fn legacy_clear_animation_data() {
    unsafe {
        for f in 0..LEGACY_SPRITEFRAME_COUNT {
            Legacy_scriptFrames[f] = DEFAULT_SPRITEFRAME;
        }
        for f in 0..LEGACY_SPRITEFRAME_COUNT {
            Legacy_animFrames[f] = DEFAULT_SPRITEFRAME;
        }
        for h in 0..LEGACY_HITBOX_COUNT {
            Legacy_hitboxList[h] = DEFAULT_HITBOX;
        }
        for a in 0..LEGACY_ANIMATION_COUNT {
            Legacy_animationList[a] = DEFAULT_SPRITEANIMATION;
        }
        for a in 0..LEGACY_ANIFILE_COUNT {
            Legacy_animationFileList[a] = DEFAULT_ANIMATIONFILE;
        }

        Legacy_scriptFrameCount = 0;
        Legacy_animFrameCount = 0;
        Legacy_animationCount = 0;
        Legacy_animationFileCount = 0;
        Legacy_hitboxCount = 0;
    }
}

#[no_mangle]
#[export_name = "Legacy_AddAnimationFile"]
pub extern "C" fn legacy_add_animation_file(filePath: *const i8) -> *const AnimationFile {
    unsafe {
        let filePathStr = CStr::from_ptr(filePath).to_str().unwrap();
        let path = "Data/Animations/".to_owned() + filePathStr + "\0";

        for a in 0..LEGACY_ANIFILE_COUNT {
            if (Legacy_animationFileList[a].fileName[0] == 0) {
                filePath.copy_to(
                    Legacy_animationFileList[a].fileName.as_mut_ptr(),
                    filePathStr.len(),
                );
                legacy_load_animation_file(path.as_ptr() as *const i8);
                Legacy_animationFileCount += 1;
                return &Legacy_animationFileList[a];
            }

            let aniFileNameStr =
                CStr::from_ptr(Legacy_animationFileList[a].fileName.as_ptr() as *const i8)
                    .to_str()
                    .unwrap();
            if (aniFileNameStr == filePathStr) {
                return &Legacy_animationFileList[a];
            }
        }
    }

    return std::ptr::null();
}

#[no_mangle]
#[export_name = "v3_ProcessObjectAnimation"]
pub extern "C" fn v3_process_object_animation(
    objectScript: &crate::scene::legacy::v3::script::ObjectScript,
    entity: &mut crate::scene::legacy::v3::object::Entity,
) {
    unsafe {
        let sprAnim = &Legacy_animationList
            [((*objectScript.animFile).aniListOffset + entity.animation as i32) as usize];

        if (entity.animationSpeed <= 0) {
            entity.animationTimer += sprAnim.speed as i32;
        } else {
            if (entity.animationSpeed > 0xF0) {
                entity.animationSpeed = 0xF0;
            }
            entity.animationTimer += entity.animationSpeed;
        }

        if (entity.animation != entity.prevAnimation) {
            entity.prevAnimation = entity.animation;
            entity.frame = 0;
            entity.animationTimer = 0;
            entity.animationSpeed = 0;
        }

        if (entity.animationTimer >= 0xF0) {
            entity.animationTimer -= 0xF0;
            entity.frame += 1;
        }

        if (entity.frame >= sprAnim.frameCount) {
            entity.frame = sprAnim.loopPoint;
        }
    }
}

#[no_mangle]
#[export_name = "v4_ProcessObjectAnimation"]
pub extern "C" fn v4_process_object_animation(
    objectScript: &crate::scene::legacy::v4::script::ObjectScript,
    entity: &mut crate::scene::legacy::v4::object::Entity,
) {
    unsafe {
        let sprAnim = &Legacy_animationList
            [((*objectScript.animFile).aniListOffset + entity.animation as i32) as usize];

        if (entity.animationSpeed <= 0) {
            entity.animationTimer += sprAnim.speed as i32;
        } else {
            if (entity.animationSpeed > 0xF0) {
                entity.animationSpeed = 0xF0;
            }
            entity.animationTimer += entity.animationSpeed;
        }

        if (entity.animation != entity.prevAnimation) {
            entity.prevAnimation = entity.animation;
            entity.frame = 0;
            entity.animationTimer = 0;
            entity.animationSpeed = 0;
        }

        if (entity.animationTimer >= 0xF0) {
            entity.animationTimer -= 0xF0;
            entity.frame += 1;
        }

        if (entity.frame >= sprAnim.frameCount) {
            entity.frame = sprAnim.loopPoint;
        }
    }
}
