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
pub enum InkEffects {
    INK_NONE,
    INK_BLEND,
    INK_ALPHA,
    INK_ADD,
    INK_SUB,
    INK_TINT,
    INK_MASKED,
    INK_UNMASKED,
}

#[repr(C)]
pub enum FlipFlags {
    FLIP_NONE,
    FLIP_X,
    FLIP_Y,
    FLIP_XY,
}

#[repr(C)]
pub enum Alignments {
    ALIGN_LEFT,
    ALIGN_RIGHT,
    ALIGN_CENTER,
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
pub const DEFAULT_SCREENINFO: ScreenInfo = ScreenInfo {
    frameBuffer: [0; SCREEN_XMAX * SCREEN_YSIZE],
    position: Vector2::new(),
    size: Vector2::new(),
    center: Vector2::new(),
    pitch: 0,
    clipBound_X1: 0,
    clipBound_Y1: 0,
    clipBound_X2: 0,
    clipBound_Y2: 0,
    waterDrawPos: 0,
};

extern "C" {
    pub fn DrawRectangle(
        x: int32,
        y: int32,
        width: int32,
        height: int32,
        color: uint32,
        alpha: int32,
        inkEffect: int32,
        screenRelative: bool32,
    );
    pub fn DrawDevString(string: *const i8, x: int32, y: int32, align: int32, color: uint32);
}

pub trait RenderDevice {
    fn Init() -> bool;
    fn CopyFrameBuffer();
    fn ProcessDimming();
    fn FlipScreen();
    fn Release(isRefresh: bool32);

    fn RefreshWindow();

    fn SetupImageTexture(width: int32, height: int32, imagePixels: *const uint8);
    fn SetupVideoTexture_YUV420(width: int32, height: int32, imagePixels: *const uint8);
    fn SetupVideoTexture_YUV422(width: int32, height: int32, imagePixels: *const uint8);
    fn SetupVideoTexture_YUV424(width: int32, height: int32, imagePixels: *const uint8);

    fn ProcessEvents() -> bool;

    fn InitFPSCap();
    fn CheckFPSCap() -> bool;
    fn UpdateFPSCap();

    // Public since it's needed for the ModAPI
    fn InitShaders() -> bool;
    fn LoadShader(fileName: *const i8, linear: bool32);
}

#[no_mangle]
pub static mut screens: [ScreenInfo; SCREEN_COUNT] = [DEFAULT_SCREENINFO; SCREEN_COUNT];
#[no_mangle]
pub static mut currentScreen: *mut ScreenInfo = std::ptr::null_mut();
