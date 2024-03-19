use crate::*;

const SURFACE_COUNT: usize = 0x40;

#[cfg(feature = "version_2")]
pub const SCREEN_COUNT: usize = 4;
#[cfg(not(feature = "version_2"))]
pub const SCREEN_COUNT: usize = 2;
pub const CAMERA_COUNT: usize = 4;

pub const DEFAULT_PIXWIDTH: usize = 424;

pub const LAYER_COUNT: usize = 8;
pub const DRAWGROUP_COUNT: usize = 16;

pub const SHADER_COUNT: usize = 0x20;

// Also for "Images" but it's a cleaner name as is
pub const RETRO_VIDEO_TEXTURE_W: usize = 1024;
pub const RETRO_VIDEO_TEXTURE_H: usize = 512;

#[repr(C)]
pub enum FlipFlags {
    FLIP_NONE,
    FLIP_X,
    FLIP_Y,
    FLIP_XY,
}

#[repr(C)]
pub struct ScreenInfo {
    // uint16 *frameBuffer;
    pub frameBuffer: [uint16; SCREEN_XMAX * SCREEN_YSIZE],
    pub position: Vector2,
    pub size: Vector2,
    pub center: Vector2,
    pub pitch: int32,
    pub clipBound_X1: int32,
    pub clipBound_Y1: int32,
    pub clipBound_X2: int32,
    pub clipBound_Y2: int32,
    pub waterDrawPos: int32,
}

#[no_mangle]
pub static mut currentScreen: *mut ScreenInfo = std::ptr::null_mut();
