use std::ffi::CStr;

use crate::*;

use self::{
    engine_core::reader::{
        close_file, init_file_info, read_int_16, read_int_32, read_int_8, read_string,
        read_string_buf, FileModes, LoadFile, Scopes, DEFAULT_FILEINFO,
    },
    scene::collision::Hitbox,
    storage::{
        allocate_storage,
        text::{gen_hash_md5, gen_hash_md5_buf, gen_hash_md5_ptr, HashMD5, RetroString},
        StorageDataSets,
    },
};

use super::sprite::LoadSpriteSheet;

const SPRFILE_COUNT: usize = 0x400;
const SPRITEFRAME_COUNT: usize = 0x400;
const SPRITEANIM_COUNT: usize = 0x40;

const FRAMEHITBOX_COUNT: usize = 0x8;

const RSDK_SIGNATURE_SPR: u32 = 0x525053; // "SPR"

#[repr(C)]
enum RotationStyles {
    ROTSTYLE_NONE,
    ROTSTYLE_FULL,
    ROTSTYLE_45DEG,
    ROTSTYLE_90DEG,
    ROTSTYLE_180DEG,
    ROTSTYLE_STATICFRAMES,
}

#[repr(C)]
pub struct SpriteFrame {
    pub sprX: int16,
    pub sprY: int16,
    pub width: int16,
    pub height: int16,
    pub pivotX: int16,
    pub pivotY: int16,
    pub duration: uint16,
    pub unicodeChar: uint16,
    pub sheetID: uint8,
    padding: [u8; 1],
    pub(crate) hitboxCount: uint8,
    pub(crate) hitboxes: [Hitbox; FRAMEHITBOX_COUNT],
}

#[repr(C)]
pub struct SpriteAnimationEntry {
    pub hash: HashMD5,
    pub frameListOffset: int32,
    pub frameCount: uint16,
    pub animationSpeed: int16,
    pub loopIndex: uint8,
    pub rotationStyle: uint8,
}

#[repr(C)]
struct SpriteAnimation {
    hash: HashMD5,
    frames: *mut SpriteFrame,
    animations: *mut SpriteAnimationEntry,
    animCount: uint16,
    scope: uint8,
}
const DEFAULT_SPRITEANIMATION: SpriteAnimation = SpriteAnimation {
    hash: [0; 4],
    frames: std::ptr::null_mut(),
    animations: std::ptr::null_mut(),
    animCount: 0,
    scope: 0,
};

#[repr(C)]
pub struct Animator {
    frames: *mut SpriteFrame,
    frameID: int32,
    animationID: int16,
    prevAnimationID: int16,
    speed: int16,
    timer: int16,
    frameDuration: int16,
    frameCount: int16,
    loopIndex: uint8,
    rotationStyle: uint8,
}

#[no_mangle]
static mut spriteAnimationList: [SpriteAnimation; SPRFILE_COUNT] =
    [DEFAULT_SPRITEANIMATION; SPRFILE_COUNT];

#[no_mangle]
#[export_name = "LoadSpriteAnimation"]
pub extern "C" fn load_sprite_animation(filePath: *const i8, scope: uint8) -> uint16 {
    if (scope == 0 || scope > Scopes::SCOPE_STAGE as u8) {
        return u16::MAX;
    }

    unsafe {
        let fullFilePath =
            ("Data/Sprites/".to_owned() + CStr::from_ptr(filePath).to_str().unwrap() + "\0")
                .as_str()
                .to_owned();
        let hash = gen_hash_md5(&fullFilePath);

        for i in 0..SPRFILE_COUNT {
            if (spriteAnimationList[i].hash == hash) {
                return i as u16;
            }
        }

        let mut id: u16 = 0;
        loop {
            if id as usize >= SPRFILE_COUNT {
                break;
            }
            if (spriteAnimationList[id as usize].scope == Scopes::SCOPE_NONE as u8) {
                break;
            }
            id += 1;
        }

        if (id as usize >= SPRFILE_COUNT) {
            return u16::MAX;
        }

        let mut nameBuffer = [[0i8; 0x20]; 0x8];
        let mut sheetIDs = [0u8; 0x18];
        sheetIDs[0] = 0;

        let mut info = DEFAULT_FILEINFO;
        init_file_info(&mut info);
        if (LoadFile(
            &mut info,
            fullFilePath.as_ptr() as *const i8,
            FileModes::FMODE_RB as u8,
        ) == true32)
        {
            let sig: uint32 = read_int_32(&mut info, false32) as u32;

            if (sig != RSDK_SIGNATURE_SPR) {
                close_file(&mut info);
                return u16::MAX;
            }

            let spr = &mut spriteAnimationList[id as usize];
            spr.scope = scope;
            spr.hash = hash;

            let frameCount: uint32 = read_int_32(&mut info, false32) as u32;
            allocate_storage(
                std::ptr::addr_of_mut!(spr.frames) as *mut *mut u8,
                frameCount * std::mem::size_of::<SpriteFrame>() as u32,
                StorageDataSets::DATASET_STG,
                false32,
            );

            let sheetCount: uint8 = read_int_8(&mut info);
            for s in 0..sheetCount {
                let path = read_string(&mut info);
                sheetIDs[s as usize] = LoadSpriteSheet(path.as_ptr() as *const i8, scope) as u8;
            }

            let hitboxCount: uint8 = read_int_8(&mut info);
            for h in 0..hitboxCount {
                read_string_buf(&mut info, nameBuffer[h as usize].as_mut_ptr());
            }

            spr.animCount = read_int_16(&mut info) as u16;
            allocate_storage(
                std::ptr::addr_of_mut!(spr.animations) as *mut *mut u8,
                spr.animCount as u32 * std::mem::size_of::<SpriteAnimationEntry>() as u32,
                StorageDataSets::DATASET_STG,
                false32,
            );

            let mut frameID: int32 = 0;
            for a in 0..spr.animCount {
                let animation = spr.animations.wrapping_add(a as usize).as_mut().unwrap();
                animation.hash = gen_hash_md5(&read_string(&mut info));

                animation.frameCount = read_int_16(&mut info) as u16;
                animation.frameListOffset = frameID;
                animation.animationSpeed = read_int_16(&mut info);
                animation.loopIndex = read_int_8(&mut info);
                animation.rotationStyle = read_int_8(&mut info);

                for f in 0..animation.frameCount {
                    let frame = spr.frames.wrapping_add(frameID as usize).as_mut().unwrap();
                    frameID += 1;

                    frame.sheetID = sheetIDs[read_int_8(&mut info) as usize];
                    frame.duration = read_int_16(&mut info) as u16;
                    frame.unicodeChar = read_int_16(&mut info) as u16;
                    frame.sprX = read_int_16(&mut info);
                    frame.sprY = read_int_16(&mut info);
                    frame.width = read_int_16(&mut info);
                    frame.height = read_int_16(&mut info);
                    frame.pivotX = read_int_16(&mut info);
                    frame.pivotY = read_int_16(&mut info);

                    frame.hitboxCount = hitboxCount;
                    for h in 0..hitboxCount {
                        frame.hitboxes[h as usize].left = read_int_16(&mut info);
                        frame.hitboxes[h as usize].top = read_int_16(&mut info);
                        frame.hitboxes[h as usize].right = read_int_16(&mut info);
                        frame.hitboxes[h as usize].bottom = read_int_16(&mut info);
                    }
                }
            }

            close_file(&mut info);

            return id;
        }
    }

    return u16::MAX;
}

#[no_mangle]
#[export_name = "SetSpriteAnimation"]
pub extern "C" fn set_sprite_animation(
    aniFrames: uint16,
    animationID: uint16,
    animator: &mut Animator,
    forceApply: bool32,
    frameID: int32,
) {
    let aniFrames = aniFrames as usize;
    if (aniFrames >= SPRFILE_COUNT || (animator as *mut Animator).is_null()) {
        if !(animator as *mut Animator).is_null() {
            animator.frames = std::ptr::null_mut();
        }
        return;
    }

    unsafe {
        let spr = &spriteAnimationList[aniFrames];
        if (animationID >= spr.animCount) {
            return;
        }

        let anim = spr.animations.wrapping_add(animationID as usize);
        let frames = spr.frames.wrapping_add((*anim).frameListOffset as usize);
        if (animator.frames == frames && forceApply == false32) {
            return;
        }

        animator.frames = frames;
        animator.timer = 0;
        animator.frameID = frameID;
        animator.frameCount = (*anim).frameCount as i16;
        animator.frameDuration = (*animator.frames.wrapping_add(frameID as usize)).duration as i16;
        animator.speed = (*anim).animationSpeed;
        animator.rotationStyle = (*anim).rotationStyle;
        animator.loopIndex = (*anim).loopIndex;
        animator.prevAnimationID = animator.animationID;
        animator.animationID = animationID as i16;
    }
}

#[no_mangle]
#[export_name = "ProcessAnimation"]
pub extern "C" fn process_animation(animator: &mut Animator) {
    if ((animator as *mut Animator).is_null() || animator.frames.is_null()) {
        return;
    }

    animator.timer += animator.speed;

    if (animator.frames as usize == 1) {
        // model anim
        while (animator.timer > animator.frameDuration) {
            animator.frameID += 1;

            animator.timer -= animator.frameDuration;
            if (animator.frameID >= animator.frameCount as i32) {
                animator.frameID = animator.loopIndex as i32;
            }
        }
    } else {
        unsafe {
            // sprite anim
            while (animator.timer > animator.frameDuration) {
                animator.frameID += 1;

                animator.timer -= animator.frameDuration;
                if (animator.frameID >= animator.frameCount as i32) {
                    animator.frameID = animator.loopIndex as i32;
                }

                animator.frameDuration =
                    (*animator.frames.wrapping_add(animator.frameID as usize)).duration as i16;
            }
        }
    }
}

#[no_mangle]
#[export_name = "GetStringWidth"]
pub extern "C" fn get_string_width(
    aniFrames: uint16,
    animID: uint16,
    string: *const RetroString,
    mut startIndex: int32,
    mut length: int32,
    spacing: int32,
) -> int32 {
    let aniFrames = aniFrames as usize;
    unsafe {
        if (aniFrames >= SPRFILE_COUNT || string.is_null() || (*string).chars.is_null()) {
            return 0;
        }

        let spr = &spriteAnimationList[aniFrames];
        if (animID < spr.animCount) {
            let anim = spr.animations.wrapping_add(animID as usize);

            startIndex = startIndex.clamp(0, (*string).length as i32 - 1);

            if (length <= 0 || length > (*string).length as i32) {
                length = (*string).length as i32;
            }

            let mut w: int32 = 0;
            for c in startIndex..length {
                let charFrame: int32 = (*(*string).chars.wrapping_add(c as usize)) as i32;
                if (charFrame < (*anim).frameCount as i32) {
                    w += (*spr
                        .frames
                        .wrapping_add(((*anim).frameListOffset + charFrame as i32) as usize))
                    .width as i32;
                    if (c + 1 >= length) {
                        return w;
                    }

                    w += spacing;
                }
            }

            return w;
        }
    }

    return 0;
}

#[no_mangle]
#[export_name = "SetSpriteString"]
pub extern "C" fn set_sprite_string(aniFrames: uint16, animID: uint16, string: &mut RetroString) {
    let aniFrames = aniFrames as usize;
    if (aniFrames >= SPRFILE_COUNT || (string as *mut RetroString).is_null()) {
        return;
    }

    unsafe {
        let spr = &spriteAnimationList[aniFrames];
        if (animID < spr.animCount) {
            let anim = spr.animations.wrapping_add(animID as usize);

            for c in 0..(string.length as usize) {
                let unicodeChar = *string.chars.wrapping_add(c);
                *string.chars.wrapping_add(c) = u16::MAX;
                for f in 0..(*anim).frameCount {
                    if ((*spr
                        .frames
                        .wrapping_add(((*anim).frameListOffset + f as i32) as usize))
                    .unicodeChar
                        == unicodeChar)
                    {
                        *string.chars.wrapping_add(c) = f;
                        break;
                    }
                }
            }
        }
    }
}

#[no_mangle]
#[export_name = "CreateSpriteAnimation"]
pub extern "C" fn create_sprite_animation(
    filename: *const i8,
    frameCount: uint32,
    animCount: uint32,
    scope: uint8,
) -> uint16 {
    if (scope == 0 || scope > Scopes::SCOPE_STAGE as u8) {
        return u16::MAX;
    }

    unsafe {
        let hash = gen_hash_md5(
            ("Data/Sprites/".to_owned() + CStr::from_ptr(filename).to_str().unwrap()).as_str(),
        );

        for i in 0..SPRFILE_COUNT {
            if spriteAnimationList[i].hash == hash {
                return i as u16;
            }
        }

        let mut id: uint16 = 0;
        loop {
            if id >= SPRFILE_COUNT as u16 {
                break;
            }

            if (spriteAnimationList[id as usize].scope == Scopes::SCOPE_NONE as u8) {
                break;
            }

            id += 1;
        }

        if (id >= SPRFILE_COUNT as u16) {
            return u16::MAX;
        }

        let spr = &mut spriteAnimationList[id as usize];
        spr.scope = scope;
        spr.hash = hash;

        allocate_storage(
            ((&mut spr.frames) as *mut *mut SpriteFrame) as *mut *mut u8,
            std::mem::size_of::<SpriteFrame>() as u32
                * u32::min(frameCount, SPRITEFRAME_COUNT as u32),
            StorageDataSets::DATASET_STG,
            true32,
        );
        allocate_storage(
            ((&mut spr.animations) as *mut *mut SpriteAnimationEntry) as *mut *mut u8,
            std::mem::size_of::<SpriteAnimationEntry>() as u32
                * u32::min(animCount, SPRITEANIM_COUNT as u32),
            StorageDataSets::DATASET_STG,
            true32,
        );

        return id;
    }
}

#[no_mangle]
#[export_name = "FindSpriteAnimation"]
pub extern "C" fn find_sprite_animation(aniFrames: uint16, name: *const i8) -> uint16 {
    let aniFrames = aniFrames as usize;
    if (aniFrames >= SPRFILE_COUNT) {
        return 0;
    }

    unsafe {
        let spr = &spriteAnimationList[aniFrames];

        let hash = gen_hash_md5_ptr(name);

        for a in 0..spr.animCount {
            if hash == (*spr.animations.wrapping_add(a as usize)).hash {
                return a;
            }
        }
    }

    return u16::MAX;
}

#[no_mangle]
#[export_name = "GetFrame"]
pub extern "C" fn get_frame(aniFrames: uint16, anim: uint16, frame: int32) -> *mut SpriteFrame {
    if (aniFrames as usize >= SPRFILE_COUNT) {
        return std::ptr::null_mut();
    }

    unsafe {
        let spr = &spriteAnimationList[aniFrames as usize];
        if (anim >= spr.animCount) {
            return std::ptr::null_mut();
        }

        return spr.frames.wrapping_add(
            (frame + (*spr.animations.wrapping_add(anim as usize)).frameListOffset) as usize,
        );
    }
}

#[no_mangle]
#[export_name = "GetHitbox"]
pub extern "C" fn get_hitbox(animator: &Animator, hitboxID: uint8) -> *mut Hitbox {
    if (!(animator as *const Animator).is_null() && !animator.frames.is_null()) {
        unsafe {
            return &mut (*animator.frames.wrapping_add(animator.frameID as usize)).hitboxes
                [hitboxID as usize & (FRAMEHITBOX_COUNT - 1)];
        }
    } else {
        return std::ptr::null_mut();
    }
}

#[no_mangle]
#[export_name = "GetFrameID"]
pub extern "C" fn get_frame_id(animator: &Animator) -> int16 {
    if (!(animator as *const Animator).is_null() && !animator.frames.is_null()) {
        unsafe {
            return (*animator.frames.wrapping_add(animator.frameID as usize)).unicodeChar as i16;
        }
    }

    return 0;
}

#[no_mangle]
#[export_name = "ClearSpriteAnimations"]
pub extern "C" fn clear_sprite_animations() {
    unsafe {
        // Unload animations
        for s in 0..SPRFILE_COUNT {
            if (spriteAnimationList[s].scope != Scopes::SCOPE_GLOBAL as u8) {
                spriteAnimationList[s] = DEFAULT_SPRITEANIMATION;
                spriteAnimationList[s].scope = Scopes::SCOPE_NONE as u8;
            }
        }
    }
}

#[no_mangle]
#[export_name = "EditSpriteAnimation"]
pub extern "C" fn edit_sprite_animation(
    aniFrames: uint16,
    animID: uint16,
    name: *const i8,
    frameOffset: int32,
    frameCount: uint16,
    animSpeed: int16,
    loopIndex: uint8,
    rotationStyle: uint8,
) {
    let aniFrames = aniFrames as usize;
    unsafe {
        if (aniFrames < SPRFILE_COUNT) {
            let spr = &spriteAnimationList[aniFrames];
            if (animID < spr.animCount) {
                let anim = spr.animations.wrapping_add(animID as usize);
                (*anim).hash = gen_hash_md5_ptr(name);
                (*anim).frameListOffset = frameOffset;
                (*anim).frameCount = frameCount;
                (*anim).animationSpeed = animSpeed;
                (*anim).loopIndex = loopIndex;
                (*anim).rotationStyle = rotationStyle;
            }
        }
    }
}
