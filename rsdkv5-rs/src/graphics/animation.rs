use crate::*;

use self::{
    scene::collision::Hitbox,
    storage::text::{HashMD5, RetroString},
};

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
struct Animator {
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
