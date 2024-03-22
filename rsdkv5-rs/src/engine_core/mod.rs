use crate::*;

use self::{audio::CHANNEL_COUNT, graphics::drawing::DRAWGROUP_COUNT};

#[cfg(feature = "version_u")]
pub mod legacy;
pub mod link;
pub mod math;
pub mod reader;

#[repr(C)]
pub struct RetroEngine {
    useExternalCode: bool32,

    devMenu: bool32,
    consoleEnabled: bool32,

    confirmFlip: bool32, // swaps A/B, used for nintendo and etc controllers
    XYFlip: bool32,      // swaps X/Y, used for nintendo and etc controllers

    focusState: uint8,
    inFocus: uint8,
    focusPausedChannel: [uint8; CHANNEL_COUNT],

    initialized: bool32,
    hardPause: bool32,

    #[cfg(feature = "version_u")]
    pub version: uint8, // determines what RSDK version to use, default to RSDKv5 since thats the "core" version

    #[cfg(feature = "version_u")]
    gamePlatform: cstr,
    #[cfg(feature = "version_u")]
    gameRenderType: cstr,
    #[cfg(feature = "version_u")]
    gameHapticSetting: cstr,

    #[cfg(feature = "version_u")]
    gameReleaseID: int32,
    #[cfg(feature = "version_u")]
    releaseType: cstr,

    storedShaderID: int32,
    storedState: int32,
    gameSpeed: int32,
    fastForwardSpeed: int32,
    frameStep: bool32,
    showPaletteOverlay: bool32,
    showUpdateRanges: uint8,
    showEntityInfo: uint8,
    drawGroupVisible: [bool32; DRAWGROUP_COUNT],

    // Image/Video support
    displayTime: double,
    videoStartDelay: double,
    imageFadeSpeed: double,
    skipCallback: Option<extern "C" fn() -> bool32>,

    streamsEnabled: bool32,
    streamVolume: float,
    soundFXVolume: float,
}

const DEFAULT_RETROENGINE: RetroEngine = RetroEngine {
    useExternalCode: true32,

    devMenu: false32,
    consoleEnabled: false32,

    confirmFlip: false32, // swaps A/B, used for nintendo and etc controllers
    XYFlip: false32,      // swaps X/Y, used for nintendo and etc controllers

    focusState: 0,
    inFocus: 0,
    focusPausedChannel: [0; CHANNEL_COUNT],

    initialized: false32,
    hardPause: false32,

    #[cfg(feature = "version_u")]
    version: 5, // determines what RSDK version to use, default to RSDKv5 since thats the "core" version

    #[cfg(feature = "version_u")]
    gamePlatform: std::ptr::null(),
    #[cfg(feature = "version_u")]
    gameRenderType: std::ptr::null(),
    #[cfg(feature = "version_u")]
    gameHapticSetting: std::ptr::null(),

    #[cfg(feature = "version_u")]
    gameReleaseID: 0,
    #[cfg(feature = "version_u")]
    releaseType: c"USE_STANDALONE".as_ptr(),

    storedShaderID: 0,
    storedState: 0,
    gameSpeed: 1,
    fastForwardSpeed: 8,
    frameStep: false32,
    showPaletteOverlay: false32,
    showUpdateRanges: 0,
    showEntityInfo: 0,
    drawGroupVisible: [false32; DRAWGROUP_COUNT],

    // Image/Video support
    displayTime: 0.0,
    videoStartDelay: 0.0,
    imageFadeSpeed: 0.0,
    skipCallback: None,

    streamsEnabled: true32,
    streamVolume: 1.0,
    soundFXVolume: 1.0,
};

#[no_mangle]
pub static mut engine: RetroEngine = DEFAULT_RETROENGINE;
