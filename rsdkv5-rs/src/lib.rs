mod engine_core;

use std::mem::size_of;

// -------------------------
// STANDARD TYPES
// -------------------------

type int8 = i8;
type uint8 = u8;
type int16 = i16;
type uint16 = u16;
type int32 = i32;
type uint32 = u32;
type float = f32;

#[repr(C)]
enum bool32 {
    False = 0,
    True = 1,
}

type color = u32;

// -------------------------
// CONSTANTS
// -------------------------

const RETRO_REVISION: i32 = 2;

const RETRO_REV01: bool = RETRO_REVISION == 1;
const RETRO_REV02: bool = RETRO_REVISION >= 2;
const RETRO_REV0U: bool = RETRO_REVISION >= 3;

const VER_100: i32 = 0;
const VER_103: i32 = 3;
const VER_105: i32 = 5;
const VER_106: i32 = 6;
const VER_107: i32 = 7;

const GAME_VERSION: i32 = VER_106;

const MANIA_USE_PLUS: bool = GAME_VERSION >= VER_105;

const SCREEN_XMAX: usize = 1280;
const SCREEN_YSIZE: usize = 240;
const SCREEN_YCENTER: usize = SCREEN_YSIZE / 2;

const LAYER_COUNT: usize = 8;
const DRAWGROUP_COUNT: usize = 16;

const SCREEN_COUNT: usize = if RETRO_REV02 { 4 } else { 2 };

const PLAYER_COUNT: usize = 4;
const CAMERA_COUNT: usize = 4;

// 0x800 scene objects, 0x40 reserved ones, and 0x100 spare slots for creation
const RESERVE_ENTITY_COUNT: usize = 0x40;
const TEMPENTITY_COUNT: usize = 0x100;
const SCENEENTITY_COUNT: usize = 0x800;
const ENTITY_COUNT: usize = RESERVE_ENTITY_COUNT + SCENEENTITY_COUNT + TEMPENTITY_COUNT;
const TEMPENTITY_START: usize = ENTITY_COUNT - TEMPENTITY_COUNT;

const TYPE_COUNT: usize = 0x100;
const TYPEGROUP_COUNT: usize = 0x104;

const CHANNEL_COUNT: usize = 0x10;

const TILE_SIZE: usize = 16;

// -------------------------
// STRUCTS
// -------------------------

#[repr(C)]
struct Vector2 {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Object {
    classID: uint16,
    active: uint8,
}

#[repr(C)]
struct Entity {
    position: Vector2,
    scale: Vector2,
    velocity: Vector2,
    updateRange: Vector2,
    angle: int32,
    alpha: int32,
    rotation: int32,
    groundVel: int32,
    zdepth: int32,
    group: uint16,
    classID: uint16,
    inRange: bool32,
    isPermanent: bool32,
    tileCollisions: int32,
    interaction: bool32,
    onGround: bool32,
    active: uint8,
    filter: uint8,
    direction: uint8,
    drawGroup: uint8,
    collisionLayers: uint8,
    collisionPlane: uint8,
    collisionMode: uint8,
    drawFX: uint8,
    inkEffect: uint8,
    visible: uint8,
    onScreen: uint8,
}

#[repr(C)]
struct RSDKSKUInfo {
    platform: int32,
    language: int32,
    region: int32,
}

// None of these besides the named 2 are even used
// and even then they're not even set in plus
#[repr(C)]
struct RSDKUnknownInfo {
    unknown1: int32,
    unknown2: int32,
    unknown3: int32,
    unknown4: int32,
    pausePress: bool32,
    unknown5: int32,
    unknown6: int32,
    unknown7: int32,
    unknown8: int32,
    unknown9: int32,
    anyKeyPress: bool32,
    unknown10: int32,
}

#[repr(C)]
struct RSDKGameInfo {
    gameTitle: [u8; 0x40],
    gameSubtitle: [u8; 0x100],
    version: [u8; 0x10],
}

#[repr(C)]
struct RSDKSceneInfo {
    entity: *mut Entity,
    listData: *mut u8,
    listCategory: *mut u8,
    timeCounter: int32,
    currentDrawGroup: int32,
    currentScreenID: int32,
    listPos: uint16,
    entitySlot: uint16,
    createSlot: uint16,
    classCount: uint16,
    inEditor: bool32,
    effectGizmo: bool32,
    debugMode: bool32,
    useGlobalScripts: bool32,
    timeEnabled: bool32,
    activeCategory: uint8,
    categoryCount: uint8,
    state: uint8,
    filter: uint8,
    milliseconds: uint8,
    seconds: uint8,
    minutes: uint8,
}

#[repr(C)]
struct InputState {
    down: bool32,
    press: bool32,
    keyMap: int32,
}

#[repr(C)]
struct RSDKControllerState {
    keyUp: InputState,
    keyDown: InputState,
    keyLeft: InputState,
    keyRight: InputState,
    keyA: InputState,
    keyB: InputState,
    keyC: InputState,
    keyX: InputState,
    keyY: InputState,
    keyZ: InputState,
    keyStart: InputState,
    keySelect: InputState,
}

#[repr(C)]
struct RSDKAnalogState {
    keyUp: InputState,
    keyDown: InputState,
    keyLeft: InputState,
    keyRight: InputState,
    keyStick: InputState,
    deadzone: float,
    hDelta: float,
    vDelta: float,
}

#[repr(C)]
struct RSDKTriggerState {
    keyBumper: InputState,
    keyTrigger: InputState,
    bumperDelta: float,
    triggerDelta: float,
}

#[repr(C)]
struct RSDKTouchInfo {
    x: [float; 0x10],
    y: [float; 0x10],
    down: [bool32; 0x10],
    count: uint8,
}

#[repr(C)]
struct RSDKScreenInfo {
    // uint16 *frameBuffer;
    frameBuffer: [uint16; SCREEN_XMAX * SCREEN_YSIZE],
    position: Vector2,
    size: Vector2,
    center: Vector2,
    pitch: int32,
    clipBound_X1: int32,
    clipBound_Y1: int32,
    clipBound_X2: int32,
    clipBound_Y2: int32,
    waterDrawPos: int32,
}

#[repr(C)]
struct EngineInfo {
    functionTable: *mut u8,
    APITable: *mut u8,

    gameInfo: *mut RSDKGameInfo,
    currentSKU: *mut RSDKSKUInfo,
    sceneInfo: *mut RSDKSceneInfo,

    controllerInfo: *mut RSDKControllerState,
    stickInfoL: *mut RSDKAnalogState,
    stickInfoR: *mut RSDKAnalogState,
    triggerInfoL: *mut RSDKTriggerState,
    triggerInfoR: *mut RSDKTriggerState,
    touchInfo: *mut RSDKTouchInfo,

    unknownInfo: *mut RSDKUnknownInfo,

    screenInfo: *mut RSDKScreenInfo,
}

#[repr(C)]
struct Matrix {
    values: [[i32; 4]; 4],
}

#[repr(C)]
struct String {
    chars: *mut uint16,
    length: uint16,
    size: uint16,
}

#[repr(C)]
struct Hitbox {
    left: int16,
    top: int16,
    right: int16,
    bottom: int16,
}

#[repr(C)]
struct SpriteFrame {
    sprX: int16,
    sprY: int16,
    width: int16,
    height: int16,
    pivotX: int16,
    pivotY: int16,
    delay: uint16,
    id: int16,
    sheetID: uint8,
}

#[repr(C)]
struct Animator {
    frames: *mut u8,
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

#[repr(C)]
struct ScrollInfo {
    tilePos: int32,
    parallaxFactor: int32,
    scrollSpeed: int32,
    scrollPos: int32,
    deform: uint8,
    unknown: uint8,
}

#[repr(C)]
struct ScanlineInfo {
    position: Vector2,
    deform: Vector2,
}

#[repr(C)]
struct TileLayer {
    type_: uint8,
    drawGroup: [uint8; 4],
    widthShift: uint8,
    heightShift: uint8,
    width: uint16,
    height: uint16,
    position: Vector2,
    parallaxFactor: int32,
    scrollSpeed: int32,
    scrollPos: int32,
    deformationOffset: int32,
    deformationOffsetW: int32,
    deformationData: [int32; 0x400],
    deformationDataW: [int32; 0x400],
    // void (*scanlineCallback)(ScanlineInfo *),
    // scanlineCallback: *const u8,
    scanlineCallback: unsafe extern "C" fn(*mut ScanlineInfo),
    scrollInfoCount: uint16,
    scrollInfo: [ScrollInfo; 0x100],
    name: [uint32; 4],
    layout: *mut uint16,
    lineScroll: *mut uint8,
}

#[repr(C)]
enum LeaderboardLoadTypes {
    LEADERBOARD_LOAD_INIT,
    LEADERBOARD_LOAD_PREV,
    LEADERBOARD_LOAD_NEXT,
}

#[repr(C)]
struct LeaderboardAvail {
    start: int32,
    length: int32,
}

#[repr(C)]
struct StatInfo {
    statID: uint8,
    name: *const u8,
    data: *mut [u8; 64],
}

#[repr(C)]
struct AchievementID {
    idPS4: uint8,     // achievement ID (PS4)
    idUnknown: int32, // achievement ID (unknown platform)
    id: *const u8,    // achievement ID (as a string, used for most platforms)
}

#[repr(C)]
struct LeaderboardID {
    idPS4: int32,      // leaderboard id (PS4)
    idUnknown: int32,  // leaderboard id (unknown platform)
    idSwitch: int32,   // leaderboard id (switch)
    idXbox: *const u8, // Xbox One Leaderboard id (making an assumption based on the MS docs)
    idPC: *const u8,   // Leaderboard id (as a string, used for PC platforms)
}

#[repr(C)]
struct LeaderboardEntry {
    username: String,
    userID: String,
    globalRank: int32,
    score: int32,
    isUser: bool32,
    status: int32,
}

// -------------------------
// ENUMS
// -------------------------

#[repr(C)]
enum GamePlatforms {
    PLATFORM_PC,
    PLATFORM_PS4,
    PLATFORM_XB1,
    PLATFORM_SWITCH,

    PLATFORM_DEV = 0xFF,
}

#[repr(C)]
enum Scopes {
    SCOPE_NONE,
    SCOPE_GLOBAL,
    SCOPE_STAGE,
}

#[repr(C)]
enum InkEffects {
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
enum DrawFX {
    FX_NONE = 0,
    FX_FLIP = 1,
    FX_ROTATE = 2,
    FX_SCALE = 4,
}

#[repr(C)]
enum FlipFlags {
    FLIP_NONE,
    FLIP_X,
    FLIP_Y,
    FLIP_XY,
}

#[repr(C)]
enum DefaultObjTypes {
    TYPE_BLANK,
}

#[repr(C)]
enum InputIDs {
    INPUT_UNASSIGNED = -2,
    INPUT_AUTOASSIGN = -1,
    INPUT_NONE = 0,
}

#[repr(C)]
enum InputSlotIDs {
    CONT_ANY,
    CONT_P1,
    CONT_P2,
    CONT_P3,
    CONT_P4,
}

#[repr(C)]
enum InputDeviceTypes {
    DEVICE_TYPE_NONE,
    DEVICE_TYPE_KEYBOARD,
    DEVICE_TYPE_CONTROLLER,
    DEVICE_TYPE_UNKNOWN,
    DEVICE_TYPE_STEAMOVERLAY,
}

#[repr(C)]
enum InputDeviceIDs {
    DEVICE_KEYBOARD,
    DEVICE_XBOX,
    DEVICE_PS4,
    DEVICE_SATURN,
    DEVICE_SWITCH_HANDHELD,
    DEVICE_SWITCH_JOY_GRIP,
    DEVICE_SWITCH_JOY_L,
    DEVICE_SWITCH_JOY_R,
    DEVICE_SWITCH_PRO,
}

#[repr(C)]
enum InputDeviceAPIs {
    DEVICE_API_NONE,
    DEVICE_API_KEYBOARD,
    DEVICE_API_XINPUT,
    DEVICE_API_RAWINPUT,
    DEVICE_API_STEAM,
}

#[repr(C)]
enum Alignments {
    ALIGN_LEFT,
    ALIGN_RIGHT,
    ALIGN_CENTER,
}

#[repr(C)]
enum PrintModes {
    PRINT_NORMAL,
    PRINT_POPUP,
    PRINT_ERROR,
    PRINT_FATAL,
}

#[repr(C)]
enum VarTypes {
    VAR_UINT8,
    VAR_UINT16,
    VAR_UINT32,
    VAR_INT8,
    VAR_INT16,
    VAR_INT32,
    VAR_ENUM,
    VAR_BOOL,
    VAR_STRING,
    VAR_VECTOR2,
    VAR_FLOAT,
    VAR_COLOR,
}

#[repr(C)]
enum DBVarTypes {
    DBVAR_UNKNOWN,
    DBVAR_BOOL,
    DBVAR_UINT8,
    DBVAR_UINT16,
    DBVAR_UINT32,
    DBVAR_UINT64, // unimplemented in RSDKv5
    DBVAR_INT8,
    DBVAR_INT16,
    DBVAR_INT32,
    DBVAR_INT64, // unimplemented in RSDKv5
    DBVAR_FLOAT,
    DBVAR_DOUBLE,  // unimplemented in RSDKv5
    DBVAR_VECTOR2, // unimplemented in RSDKv5
    DBVAR_VECTOR3, // unimplemented in RSDKv5
    DBVAR_VECTOR4, // unimplemented in RSDKv5
    DBVAR_COLOR,
    DBVAR_STRING,
    DBVAR_HASHMD5, // unimplemented in RSDKv5
}

#[repr(C)]
enum ViewableVarTypes {
    VIEWVAR_INVALID,
    VIEWVAR_BOOL,
    VIEWVAR_UINT8,
    VIEWVAR_UINT16,
    VIEWVAR_UINT32,
    VIEWVAR_INT8,
    VIEWVAR_INT16,
    VIEWVAR_INT32,
}

#[repr(C)]
enum ActiveFlags {
    ACTIVE_NEVER,
    ACTIVE_ALWAYS,
    ACTIVE_NORMAL,
    ACTIVE_PAUSED,
    ACTIVE_BOUNDS,
    ACTIVE_XBOUNDS,
    ACTIVE_YBOUNDS,
    ACTIVE_RBOUNDS,

    // Not really even a real active value, but some objects set their active states to this so here it is I suppose
    ACTIVE_DISABLED = 0xFF,
}

#[repr(C)]
enum RotationSyles {
    ROTSTYLE_NONE,
    ROTSTYLE_FULL,
    ROTSTYLE_45DEG,
    ROTSTYLE_90DEG,
    ROTSTYLE_180DEG,
    ROTSTYLE_STATICFRAMES,
}

#[repr(C)]
enum LayerTypes {
    LAYER_HSCROLL,
    LAYER_VSCROLL,
    LAYER_ROTOZOOM,
    LAYER_BASIC,
}

#[repr(C)]
enum CModes {
    CMODE_FLOOR,
    CMODE_LWALL,
    CMODE_ROOF,
    CMODE_RWALL,
}

#[repr(C)]
enum CSides {
    C_NONE,
    C_TOP,
    C_LEFT,
    C_RIGHT,
    C_BOTTOM,
}

#[repr(C)]
enum TileCollisionModes {
    TILECOLLISION_NONE, // no tile collisions
    TILECOLLISION_DOWN, // downwards tile collisions
}

#[repr(C)]
enum Scene3DDrawTypes {
    S3D_WIREFRAME,
    S3D_SOLIDCOLOR,

    S3D_UNUSED_1,
    S3D_UNUSED_2,

    S3D_WIREFRAME_SHADED,
    S3D_SOLIDCOLOR_SHADED,

    S3D_SOLIDCOLOR_SHADED_BLENDED,

    S3D_WIREFRAME_SCREEN,
    S3D_SOLIDCOLOR_SCREEN,

    S3D_WIREFRAME_SHADED_SCREEN,
    S3D_SOLIDCOLOR_SHADED_SCREEN,

    S3D_SOLIDCOLOR_SHADED_BLENDED_SCREEN,
}

#[repr(C)]
enum VideoSettingsValues {
    VIDEOSETTING_WINDOWED,
    VIDEOSETTING_BORDERED,
    VIDEOSETTING_EXCLUSIVEFS,
    VIDEOSETTING_VSYNC,
    VIDEOSETTING_TRIPLEBUFFERED,
    VIDEOSETTING_WINDOW_WIDTH,
    VIDEOSETTING_WINDOW_HEIGHT,
    VIDEOSETTING_FSWIDTH,
    VIDEOSETTING_FSHEIGHT,
    VIDEOSETTING_REFRESHRATE,
    VIDEOSETTING_SHADERSUPPORT,
    VIDEOSETTING_SHADERID,
    VIDEOSETTING_SCREENCOUNT,
    VIDEOSETTING_DIMTIMER,
    VIDEOSETTING_STREAMSENABLED,
    VIDEOSETTING_STREAM_VOL,
    VIDEOSETTING_SFX_VOL,
    VIDEOSETTING_LANGUAGE,
    VIDEOSETTING_STORE,
    VIDEOSETTING_RELOAD,
    VIDEOSETTING_CHANGED,
    VIDEOSETTING_WRITE,
}

#[repr(C)]
enum TypeGroups {
    GROUP_ALL = 0,

    GROUP_CUSTOM0 = TYPE_COUNT as isize,
    GROUP_CUSTOM1,
    GROUP_CUSTOM2,
    GROUP_CUSTOM3,
}

#[repr(C)]
enum StatusCodes {
    STATUS_NONE = 0,
    STATUS_CONTINUE = 100,
    STATUS_OK = 200,
    STATUS_FORBIDDEN = 403,
    STATUS_NOTFOUND = 404,
    STATUS_ERROR = 500,
    STATUS_NOWIFI = 503,
    STATUS_TIMEOUT = 504,
    STATUS_CORRUPT = 505,
    STATUS_NOSPACE = 506,
}

#[repr(C)]
enum GameRegions {
    REGION_US,
    REGION_JP,
    REGION_EU,
}

#[repr(C)]
enum GameLanguages {
    LANGUAGE_EN,
    LANGUAGE_FR,
    LANGUAGE_IT,
    LANGUAGE_GE,
    LANGUAGE_SP,
    LANGUAGE_JP,
    LANGUAGE_KO,
    LANGUAGE_SC,
    LANGUAGE_TC,
}

#[repr(C)]
enum EngineStates {
    ENGINESTATE_LOAD,
    ENGINESTATE_REGULAR,
    ENGINESTATE_PAUSED,
    ENGINESTATE_FROZEN,
    ENGINESTATE_STEPOVER = 4,
    ENGINESTATE_DEVMENU = 8,
    ENGINESTATE_VIDEOPLAYBACK,
    ENGINESTATE_SHOWIMAGE,
    ENGINESTATE_ERRORMSG,
    ENGINESTATE_ERRORMSG_FATAL,
    ENGINESTATE_NONE,
}

// see: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
// for value list & descriptions
#[repr(C)]
enum KeyMappings {
    KEYMAP_AUTO_MAPPING = -1,
    KEYMAP_NO_MAPPING = 0,
    KEYMAP_LBUTTON = 0x01,
    KEYMAP_RBUTTON = 0x02,
    KEYMAP_CANCEL = 0x03,
    KEYMAP_MBUTTON = 0x04,
    KEYMAP_XBUTTON1 = 0x05,
    KEYMAP_XBUTTON2 = 0x06,
    KEYMAP_BACK = 0x08,
    KEYMAP_TAB = 0x09,
    KEYMAP_CLEAR = 0x0C,
    KEYMAP_RETURN = 0x0D,
    KEYMAP_SHIFT = 0x10,
    KEYMAP_CONTROL = 0x11,
    KEYMAP_MENU = 0x12,
    KEYMAP_PAUSE = 0x13,
    KEYMAP_CAPITAL = 0x14,
    KEYMAP_KANA = 0x15,
    // uh sorry
    // KEYMAP_HANGEUL = 0x15,
    // KEYMAP_HANGUL = 0x15,
    KEYMAP_JUNJA = 0x17,
    KEYMAP_FINAL = 0x18,
    // i'm trying my best here
    KEYMAP_HANJA = 0x19,
    // KEYMAP_KANJI = 0x19,
    KEYMAP_ESCAPE = 0x1B,
    KEYMAP_CONVERT = 0x1C,
    KEYMAP_NONCONVERT = 0x1D,
    KEYMAP_ACCEPT = 0x1E,
    KEYMAP_MODECHANGE = 0x1F,
    KEYMAP_SPACE = 0x20,
    KEYMAP_PRIOR = 0x21,
    KEYMAP_NEXT = 0x22,
    KEYMAP_END = 0x23,
    KEYMAP_HOME = 0x24,
    KEYMAP_LEFT = 0x25,
    KEYMAP_UP = 0x26,
    KEYMAP_RIGHT = 0x27,
    KEYMAP_DOWN = 0x28,
    KEYMAP_SELECT = 0x29,
    KEYMAP_PRINT = 0x2A,
    KEYMAP_EXECUTE = 0x2B,
    KEYMAP_SNAPSHOT = 0x2C,
    KEYMAP_INSERT = 0x2D,
    KEYMAP_DELETE = 0x2E,
    KEYMAP_HELP = 0x2F,
    KEYMAP_0 = 0x30,
    KEYMAP_1 = 0x31,
    KEYMAP_2 = 0x32,
    KEYMAP_3 = 0x33,
    KEYMAP_4 = 0x34,
    KEYMAP_5 = 0x35,
    KEYMAP_6 = 0x36,
    KEYMAP_7 = 0x37,
    KEYMAP_8 = 0x38,
    KEYMAP_9 = 0x39,
    KEYMAP_A = 0x41,
    KEYMAP_B = 0x42,
    KEYMAP_C = 0x43,
    KEYMAP_D = 0x44,
    KEYMAP_E = 0x45,
    KEYMAP_F = 0x46,
    KEYMAP_G = 0x47,
    KEYMAP_H = 0x48,
    KEYMAP_I = 0x49,
    KEYMAP_J = 0x4A,
    KEYMAP_K = 0x4B,
    KEYMAP_L = 0x4C,
    KEYMAP_M = 0x4D,
    KEYMAP_N = 0x4E,
    KEYMAP_O = 0x4F,
    KEYMAP_P = 0x50,
    KEYMAP_Q = 0x51,
    KEYMAP_R = 0x52,
    KEYMAP_S = 0x53,
    KEYMAP_T = 0x54,
    KEYMAP_U = 0x55,
    KEYMAP_V = 0x56,
    KEYMAP_W = 0x57,
    KEYMAP_X = 0x58,
    KEYMAP_Y = 0x59,
    KEYMAP_Z = 0x5A,
    KEYMAP_LWIN = 0x5B,
    KEYMAP_RWIN = 0x5C,
    KEYMAP_APPS = 0x5D,
    KEYMAP_SLEEP = 0x5F,
    KEYMAP_NUMPAD0 = 0x60,
    KEYMAP_NUMPAD1 = 0x61,
    KEYMAP_NUMPAD2 = 0x62,
    KEYMAP_NUMPAD3 = 0x63,
    KEYMAP_NUMPAD4 = 0x64,
    KEYMAP_NUMPAD5 = 0x65,
    KEYMAP_NUMPAD6 = 0x66,
    KEYMAP_NUMPAD7 = 0x67,
    KEYMAP_NUMPAD8 = 0x68,
    KEYMAP_NUMPAD9 = 0x69,
    KEYMAP_MULTIPLY = 0x6A,
    KEYMAP_ADD = 0x6B,
    KEYMAP_SEPARATOR = 0x6C,
    KEYMAP_SUBTRACT = 0x6D,
    KEYMAP_DECIMAL = 0x6E,
    KEYMAP_DIVIDE = 0x6F,
    KEYMAP_F1 = 0x70,
    KEYMAP_F2 = 0x71,
    KEYMAP_F3 = 0x72,
    KEYMAP_F4 = 0x73,
    KEYMAP_F5 = 0x74,
    KEYMAP_F6 = 0x75,
    KEYMAP_F7 = 0x76,
    KEYMAP_F8 = 0x77,
    KEYMAP_F9 = 0x78,
    KEYMAP_F10 = 0x79,
    KEYMAP_F11 = 0x7A,
    KEYMAP_F12 = 0x7B,
    KEYMAP_F13 = 0x7C,
    KEYMAP_F14 = 0x7D,
    KEYMAP_F15 = 0x7E,
    KEYMAP_F16 = 0x7F,
    KEYMAP_F17 = 0x80,
    KEYMAP_F18 = 0x81,
    KEYMAP_F19 = 0x82,
    KEYMAP_F20 = 0x83,
    KEYMAP_F21 = 0x84,
    KEYMAP_F22 = 0x85,
    KEYMAP_F23 = 0x86,
    KEYMAP_F24 = 0x87,
    KEYMAP_NAVIGATION_VIEW = 0x88,
    KEYMAP_NAVIGATION_MENU = 0x89,
    KEYMAP_NAVIGATION_UP = 0x8A,
    KEYMAP_NAVIGATION_DOWN = 0x8B,
    KEYMAP_NAVIGATION_LEFT = 0x8C,
    KEYMAP_NAVIGATION_RIGHT = 0x8D,
    KEYMAP_NAVIGATION_ACCEPT = 0x8E,
    KEYMAP_NAVIGATION_CANCEL = 0x8F,
    KEYMAP_NUMLOCK = 0x90,
    KEYMAP_SCROLL = 0x91,
    // you'll have to speak with mx. rust about that
    KEYMAP_OEM_NEC_EQUAL = 0x92,
    // KEYMAP_OEM_FJ_JISHO = 0x92,
    KEYMAP_OEM_FJ_MASSHOU = 0x93,
    KEYMAP_OEM_FJ_TOUROKU = 0x94,
    KEYMAP_OEM_FJ_LOYA = 0x95,
    KEYMAP_OEM_FJ_ROYA = 0x96,
    KEYMAP_LSHIFT = 0xA0,
    KEYMAP_RSHIFT = 0xA1,
    KEYMAP_LCONTROL = 0xA2,
    KEYMAP_RCONTROL = 0xA3,
    KEYMAP_LMENU = 0xA4,
    KEYMAP_RMENU = 0xA5,
    KEYMAP_BROWSER_BACK = 0xA6,
    KEYMAP_BROWSER_FORWARD = 0xA7,
    KEYMAP_BROWSER_REFRESH = 0xA8,
    KEYMAP_BROWSER_STOP = 0xA9,
    KEYMAP_BROWSER_SEARCH = 0xAA,
    KEYMAP_BROWSER_FAVORITES = 0xAB,
    KEYMAP_BROWSER_HOME = 0xAC,
    KEYMAP_VOLUME_MUTE = 0xAD,
    KEYMAP_VOLUME_DOWN = 0xAE,
    KEYMAP_VOLUME_UP = 0xAF,
    KEYMAP_MEDIA_NEXT_TRACK = 0xB0,
    KEYMAP_MEDIA_PREV_TRACK = 0xB1,
    KEYMAP_MEDIA_STOP = 0xB2,
    KEYMAP_MEDIA_PLAY_PAUSE = 0xB3,
    KEYMAP_LAUNCH_MAIL = 0xB4,
    KEYMAP_LAUNCH_MEDIA_SELECT = 0xB5,
    KEYMAP_LAUNCH_APP1 = 0xB6,
    KEYMAP_LAUNCH_APP2 = 0xB7,
    KEYMAP_OEM_1 = 0xBA,
    KEYMAP_OEM_PLUS = 0xBB,
    KEYMAP_OEM_COMMA = 0xBC,
    KEYMAP_OEM_MINUS = 0xBD,
    KEYMAP_OEM_PERIOD = 0xBE,
    KEYMAP_OEM_2 = 0xBF,
    KEYMAP_OEM_3 = 0xC0,
    KEYMAP_GAMEPAD_A = 0xC3,
    KEYMAP_GAMEPAD_B = 0xC4,
    KEYMAP_GAMEPAD_X = 0xC5,
    KEYMAP_GAMEPAD_Y = 0xC6,
    KEYMAP_GAMEPAD_RIGHT_SHOULDER = 0xC7,
    KEYMAP_GAMEPAD_LEFT_SHOULDER = 0xC8,
    KEYMAP_GAMEPAD_LEFT_TRIGGER = 0xC9,
    KEYMAP_GAMEPAD_RIGHT_TRIGGER = 0xCA,
    KEYMAP_GAMEPAD_DPAD_UP = 0xCB,
    KEYMAP_GAMEPAD_DPAD_DOWN = 0xCC,
    KEYMAP_GAMEPAD_DPAD_LEFT = 0xCD,
    KEYMAP_GAMEPAD_DPAD_RIGHT = 0xCE,
    KEYMAP_GAMEPAD_MENU = 0xCF,
    KEYMAP_GAMEPAD_VIEW = 0xD0,
    KEYMAP_GAMEPAD_LEFT_THUMBSTICK_BUTTON = 0xD1,
    KEYMAP_GAMEPAD_RIGHT_THUMBSTICK_BUTTON = 0xD2,
    KEYMAP_GAMEPAD_LEFT_THUMBSTICK_UP = 0xD3,
    KEYMAP_GAMEPAD_LEFT_THUMBSTICK_DOWN = 0xD4,
    KEYMAP_GAMEPAD_LEFT_THUMBSTICK_RIGHT = 0xD5,
    KEYMAP_GAMEPAD_LEFT_THUMBSTICK_LEFT = 0xD6,
    KEYMAP_GAMEPAD_RIGHT_THUMBSTICK_UP = 0xD7,
    KEYMAP_GAMEPAD_RIGHT_THUMBSTICK_DOWN = 0xD8,
    KEYMAP_GAMEPAD_RIGHT_THUMBSTICK_RIGHT = 0xD9,
    KEYMAP_GAMEPAD_RIGHT_THUMBSTICK_LEFT = 0xDA,
    KEYMAP_OEM_4 = 0xDB,
    KEYMAP_OEM_5 = 0xDC,
    KEYMAP_OEM_6 = 0xDD,
    KEYMAP_OEM_7 = 0xDE,
    KEYMAP_OEM_8 = 0xDF,
    KEYMAP_OEM_AX = 0xE1,
    KEYMAP_OEM_102 = 0xE2,
    KEYMAP_ICO_HELP = 0xE3,
    KEYMAP_ICO_00 = 0xE4,
    KEYMAP_PROCESSKEY = 0xE5,
    KEYMAP_ICO_CLEAR = 0xE6,
    KEYMAP_PACKET = 0xE7,
    KEYMAP_OEM_RESET = 0xE9,
    KEYMAP_OEM_JUMP = 0xEA,
    KEYMAP_OEM_PA1 = 0xEB,
    KEYMAP_OEM_PA2 = 0xEC,
    KEYMAP_OEM_PA3 = 0xED,
    KEYMAP_OEM_WSCTRL = 0xEE,
    KEYMAP_OEM_CUSEL = 0xEF,
    KEYMAP_OEM_ATTN = 0xF0,
    KEYMAP_OEM_FINISH = 0xF1,
    KEYMAP_OEM_COPY = 0xF2,
    KEYMAP_OEM_AUTO = 0xF3,
    KEYMAP_OEM_ENLW = 0xF4,
    KEYMAP_OEM_BACKTAB = 0xF5,
    KEYMAP_ATTN = 0xF6,
    KEYMAP_CRSEL = 0xF7,
    KEYMAP_EXSEL = 0xF8,
    KEYMAP_EREOF = 0xF9,
    KEYMAP_PLAY = 0xFA,
    KEYMAP_ZOOM = 0xFB,
    KEYMAP_NONAME = 0xFC,
    KEYMAP_PA1 = 0xFD,
    KEYMAP_OEM_CLEAR = 0xFE,
}

// -------------------------
// FUNCTION TABLES
// -------------------------

// API Table
struct APIFunctionTable {
    // API Core
    GetUserLanguage: extern "C" fn() -> i32,
    GetConfirmButtonFlip: extern "C" fn() -> bool32,
    ExitGame: extern "C" fn(),
    LaunchManual: extern "C" fn(),
    IsOverlayEnabled: extern "C" fn(u32) -> bool32,
    /*
        bool32 (*CheckDLC)(int32 dlc);
        bool32 (*ShowExtensionOverlay)(int32 overlay);

        // Achievements
        void (*TryUnlockAchievement)(AchievementID *id);
        bool32 (*GetAchievementsEnabled)(void);
        void (*SetAchievementsEnabled)(bool32 enabled);

        // Leaderboards
        void (*InitLeaderboards)(void);
        void (*FetchLeaderboard)(LeaderboardID *leaderboard, bool32 isUser);
        void (*TrackScore)(LeaderboardID *leaderboard, int32 score, void (*callback)(bool32 success, int32 rank));
        int32 (*GetLeaderboardsStatus)(void);
        LeaderboardAvail (*LeaderboardEntryViewSize)(void);
        LeaderboardAvail (*LeaderboardEntryLoadSize)(void);
        void (*LoadLeaderboardEntries)(int32 start, uint32 end, int32 type);
        void (*ResetLeaderboardInfo)(void);
        LeaderboardEntry *(*ReadLeaderboardEntry)(uint32 entryID);

        // Rich Presence
        void (*SetRichPresence)(int32 id, String *text);

        // Stats
        void (*TryTrackStat)(StatInfo *stat);
        bool32 (*GetStatsEnabled)(void);
        void (*SetStatsEnabled)(bool32 enabled);

        // Authorization
        void (*ClearPrerollErrors)(void);
        void (*TryAuth)(void);
        int32 (*GetUserAuthStatus)(void);
        bool32 (*GetUsername)(String *userName);

        // Storage
        void (*TryInitStorage)(void);
        int32 (*GetStorageStatus)(void);
        int32 (*GetSaveStatus)(void);
        void (*ClearSaveStatus)(void);
        void (*SetSaveStatusContinue)(void);
        void (*SetSaveStatusOK)(void);
        void (*SetSaveStatusForbidden)(void);
        void (*SetSaveStatusError)(void);
        void (*SetNoSave)(bool32 noSave);
        bool32 (*GetNoSave)(void);

        // User File Management
        void (*LoadUserFile)(const char *name, void *buffer, uint32 size, void (*callback)(int32 status)); // load user file from game dir
        void (*SaveUserFile)(const char *name, void *buffer, uint32 size, void (*callback)(int32 status), bool32 compressed); // save user file to game dir
        void (*DeleteUserFile)(const char *name, void (*callback)(int32 status)); // delete user file from game dir

        // User DBs
        uint16 (*InitUserDB)(const char *name, ...);
        uint16 (*LoadUserDB)(const char *filename, void (*callback)(int32 status));
        bool32 (*SaveUserDB)(uint16 tableID, void (*callback)(int32 status));
        void (*ClearUserDB)(uint16 tableID);
        void (*ClearAllUserDBs)(void);
        uint16 (*SetupUserDBRowSorting)(uint16 tableID);
        bool32 (*GetUserDBRowsChanged)(uint16 tableID);
        int32 (*AddRowSortFilter)(uint16 tableID, int32 type, const char *name, void *value);
        int32 (*SortDBRows)(uint16 tableID, int32 type, const char *name, bool32 sortAscending);
        int32 (*GetSortedUserDBRowCount)(uint16 tableID);
        int32 (*GetSortedUserDBRowID)(uint16 tableID, uint16 row);
        int32 (*AddUserDBRow)(uint16 tableID);
        bool32 (*SetUserDBValue)(uint16 tableID, uint32 row, int32 type, const char *name, void *value);
        bool32 (*GetUserDBValue)(uint16 tableID, uint32 row, int32 type, const char *name, void *value);
        uint32 (*GetUserDBRowUUID)(uint16 tableID, uint16 row);
        uint16 (*GetUserDBRowByID)(uint16 tableID, uint32 uuid);
        void (*GetUserDBRowCreationTime)(uint16 tableID, uint16 row, char *buffer, size_t bufferSize, const char *format);
        bool32 (*RemoveDBRow)(uint16 tableID, uint16 row);
        bool32 (*RemoveAllDBRows)(uint16 tableID);
    */
}

// Function Table
struct RSDKFunctionTable {
    /*
    // Registration
    void (*RegisterGlobalVariables)(void **globals, int32 size);
    void (*RegisterObject)(void **staticVars, const char *name, uint32 entityClassSize, uint32 staticClassSize, void (*update)(void),
                           void (*lateUpdate)(void), void (*staticUpdate)(void), void (*draw)(void), void (*create)(void *), void (*stageLoad)(void),
                           void (*editorDraw)(void), void (*editorLoad)(void), void (*serialize)(void));
    void (*RegisterStaticVariables)(void **varClass, const char *name, uint32 classSize);

    // Entities & Objects
    bool32 (*GetActiveEntities)(uint16 group, void **entity);
    bool32 (*GetAllEntities)(uint16 classID, void **entity);
    void (*BreakForeachLoop)(void);
    void (*SetEditableVar)(uint8 type, const char *name, uint8 classID, int32 offset);
    void *(*GetEntity)(uint16 slot);
    int32 (*GetEntitySlot)(void *entity);
    int32 (*GetEntityCount)(uint16 classID, bool32 isActive);
    int32 (*GetDrawListRefSlot)(uint8 drawGroup, uint16 listPos);
    void *(*GetDrawListRef)(uint8 drawGroup, uint16 listPos);
    void (*ResetEntity)(void *entity, uint16 classID, void *data);
    void (*ResetEntitySlot)(uint16 slot, uint16 classID, void *data);
    Entity *(*CreateEntity)(uint16 classID, void *data, int32 x, int32 y);
    void (*CopyEntity)(void *destEntity, void *srcEntity, bool32 clearSrcEntity);
    bool32 (*CheckOnScreen)(void *entity, Vector2 *range);
    bool32 (*CheckPosOnScreen)(Vector2 *position, Vector2 *range);
    void (*AddDrawListRef)(uint8 drawGroup, uint16 entitySlot);
    void (*SwapDrawListEntries)(uint8 drawGroup, uint16 slot1, uint16 slot2, uint16 count);
    void (*SetDrawGroupProperties)(uint8 drawGroup, bool32 sorted, void (*hookCB)(void));

    // Scene Management
    void (*SetScene)(const char *categoryName, const char *sceneName);
    void (*SetEngineState)(uint8 state);
    void (*ForceHardReset)(bool32 shouldHardReset);
    bool32 (*CheckValidScene)(void);
    bool32 (*CheckSceneFolder)(const char *folderName);
    void (*LoadScene)(void);
    int32 (*FindObject)(const char *name);

    // Cameras
    void (*ClearCameras)(void);
    void (*AddCamera)(Vector2 *targetPos, int32 offsetX, int32 offsetY, bool32 worldRelative);

    // Window/Video Settings
    int32 (*GetVideoSetting)(int32 id);
    void (*SetVideoSetting)(int32 id, int32 value);
    void (*UpdateWindow)(void);

    // Math
    int32 (*Sin1024)(int32 angle);
    int32 (*Cos1024)(int32 angle);
    int32 (*Tan1024)(int32 angle);
    int32 (*ASin1024)(int32 angle);
    int32 (*ACos1024)(int32 angle);
    int32 (*Sin512)(int32 angle);
    int32 (*Cos512)(int32 angle);
    int32 (*Tan512)(int32 angle);
    int32 (*ASin512)(int32 angle);
    int32 (*ACos512)(int32 angle);
    int32 (*Sin256)(int32 angle);
    int32 (*Cos256)(int32 angle);
    int32 (*Tan256)(int32 angle);
    int32 (*ASin256)(int32 angle);
    int32 (*ACos256)(int32 angle);
    int32 (*Rand)(int32 min, int32 max);
    int32 (*RandSeeded)(int32 min, int32 max, int32 *seed);
    void (*SetRandSeed)(int32 seed);
    uint8 (*ATan2)(int32 x, int32 y);

    // Matrices
    void (*SetIdentityMatrix)(Matrix *matrix);
    void (*MatrixMultiply)(Matrix *dest, Matrix *matrixA, Matrix *matrixB);
    void (*MatrixTranslateXYZ)(Matrix *matrix, int32 x, int32 y, int32 z, bool32 setIdentity);
    void (*MatrixScaleXYZ)(Matrix *matrix, int32 x, int32 y, int32 z);
    void (*MatrixRotateX)(Matrix *matrix, int16 angle);
    void (*MatrixRotateY)(Matrix *matrix, int16 angle);
    void (*MatrixRotateZ)(Matrix *matrix, int16 angle);
    void (*MatrixRotateXYZ)(Matrix *matrix, int16 x, int16 y, int16 z);
    void (*MatrixInverse)(Matrix *dest, Matrix *matrix);
    void (*MatrixCopy)(Matrix *matDest, Matrix *matSrc);

    // Strings
    void (*InitString)(String *string, const char *text, uint32 textLength);
    void (*CopyString)(String *dst, String *src);
    void (*SetString)(String *string, const char *text);
    void (*AppendString)(String *string, String *appendString);
    void (*AppendText)(String *string, const char *appendText);
    void (*LoadStringList)(String *stringList, const char *filePath, uint32 charSize);
    bool32 (*SplitStringList)(String *splitStrings, String *stringList, int32 startStringID, int32 stringCount);
    void (*GetCString)(char *destChars, String *string);
    bool32 (*CompareStrings)(String *string1, String *string2, bool32 exactMatch);

    // Screens & Displays
    void (*GetDisplayInfo)(int32 *displayID, int32 *width, int32 *height, int32 *refreshRate, char *text);
    void (*GetWindowSize)(int32 *width, int32 *height);
    int32 (*SetScreenSize)(uint8 screenID, uint16 width, uint16 height);
    void (*SetClipBounds)(uint8 screenID, int32 x1, int32 y1, int32 x2, int32 y2);
    void (*SetScreenVertices)(uint8 startVert2P_S1, uint8 startVert2P_S2, uint8 startVert3P_S1, uint8 startVert3P_S2, uint8 startVert3P_S3);

    // Spritesheets
    uint16 (*LoadSpriteSheet)(const char *filePath, uint8 scope);

    // Palettes & Colors
    void (*SetTintLookupTable)(uint16 *lookupTable);
    void (*SetPaletteMask)(color maskColor);
    void (*SetPaletteEntry)(uint8 bankID, uint8 index, uint32 color);
    color (*GetPaletteEntry)(uint8 bankID, uint8 index);
    void (*SetActivePalette)(uint8 newActiveBank, int32 startLine, int32 endLine);
    void (*CopyPalette)(uint8 sourceBank, uint8 srcBankStart, uint8 destinationBank, uint8 destBankStart, uint16 count);
    void (*LoadPalette)(uint8 bankID, const char *path, uint16 disabledRows);
    void (*RotatePalette)(uint8 bankID, uint8 startIndex, uint8 endIndex, bool32 right);
    void (*SetLimitedFade)(uint8 destBankID, uint8 srcBankA, uint8 srcBankB, int16 blendAmount, int32 startIndex, int32 endIndex);
    void (*BlendColors)(uint8 destBankID, color *srcColorsA, color *srcColorsB, int32 blendAmount, int32 startIndex, int32 count);

    // Drawing
    void (*DrawRect)(int32 x, int32 y, int32 width, int32 height, uint32 color, int32 alpha, int32 inkEffect, bool32 screenRelative);
    void (*DrawLine)(int32 x1, int32 y1, int32 x2, int32 y2, uint32 color, int32 alpha, int32 inkEffect, bool32 screenRelative);
    void (*DrawCircle)(int32 x, int32 y, int32 radius, uint32 color, int32 alpha, int32 inkEffect, bool32 screenRelative);
    void (*DrawCircleOutline)(int32 x, int32 y, int32 innerRadius, int32 outerRadius, uint32 color, int32 alpha, int32 inkEffect,
                              bool32 screenRelative);
    void (*DrawFace)(Vector2 *vertices, int32 vertCount, int32 r, int32 g, int32 b, int32 alpha, int32 inkEffect);
    void (*DrawBlendedFace)(Vector2 *vertices, color *vertColors, int32 vertCount, int32 alpha, int32 inkEffect);
    void (*DrawSprite)(Animator *animator, Vector2 *position, bool32 screenRelative);
    void (*DrawDeformedSprite)(uint16 sheetID, int32 inkEffect, bool32 screenRelative);
    void (*DrawText)(Animator *animator, Vector2 *position, String *string, int32 endFrame, int32 textLength, int32 align, int32 spacing, void *unused,
                     Vector2 *charOffsets, bool32 screenRelative);
    void (*DrawTile)(uint16 *tiles, int32 countX, int32 countY, Vector2 *position, Vector2 *offset, bool32 screenRelative);
    void (*CopyTile)(uint16 dest, uint16 src, uint16 count);
    void (*DrawAniTiles)(uint16 sheetID, uint16 tileIndex, uint16 srcX, uint16 srcY, uint16 width, uint16 height);
    void (*FillScreen)(uint32 color, int32 alphaR, int32 alphaG, int32 alphaB);

    // Meshes & 3D Scenes
    uint16 (*LoadMesh)(const char *filename, uint8 scope);
    uint16 (*Create3DScene)(const char *identifier, uint16 faceCount, uint8 scope);
    void (*Prepare3DScene)(uint16 sceneIndex);
    void (*SetDiffuseColor)(uint16 sceneIndex, uint8 x, uint8 y, uint8 z);
    void (*SetDiffuseIntensity)(uint16 sceneIndex, uint8 x, uint8 y, uint8 z);
    void (*SetSpecularIntensity)(uint16 sceneIndex, uint8 x, uint8 y, uint8 z);
    void (*AddModelTo3DScene)(uint16 modelFrames, uint16 sceneIndex, uint8 drawMode, Matrix *matWorld, Matrix *matView, color color);
    void (*SetModelAnimation)(uint16 modelFrames, Animator *animator, int16 speed, uint8 loopIndex, bool32 forceApply, int16 frameID);
    void (*AddMeshFrameTo3DScene)(uint16 modelFrames, uint16 sceneIndex, Animator *animator, uint8 drawMode, Matrix *matWorld, Matrix *matView,
                                  color color);
    void (*Draw3DScene)(uint16 sceneIndex);

    // Sprite Animations & Frames
    uint16 (*LoadSpriteAnimation)(const char *filePath, uint8 scope);
    uint16 (*CreateSpriteAnimation)(const char *filePath, uint32 frameCount, uint32 listCount, uint8 scope);
    void (*SetSpriteAnimation)(uint16 aniFrames, uint16 listID, Animator *animator, bool32 forceApply, int32 frameID);
    void (*EditSpriteAnimation)(uint16 aniFrames, uint16 listID, const char *name, int32 frameOffset, uint16 frameCount, int16 speed, uint8 loopIndex,
                                uint8 rotationStyle);
    void (*SetSpriteString)(uint16 aniFrames, uint16 listID, String *string);
    uint16 (*FindSpriteAnimation)(uint16 aniFrames, const char *name);
    SpriteFrame *(*GetFrame)(uint16 aniFrames, uint16 listID, int32 frameID);
    Hitbox *(*GetHitbox)(Animator *animator, uint8 hitboxID);
    int16 (*GetFrameID)(Animator *animator);
    int32 (*GetStringWidth)(uint16 aniFrames, uint16 listID, String *string, int32 startIndex, int32 length, int32 spacing);
    void (*ProcessAnimation)(Animator *animator);

    // Tile Layers
    uint16 (*GetTileLayerID)(const char *name);
    TileLayer *(*GetTileLayer)(uint16 layerID);
    void (*GetLayerSize)(uint16 layer, Vector2 *size, bool32 usePixelUnits);
    uint16 (*GetTile)(uint16 layer, int32 x, int32 y);
    void (*SetTile)(uint16 layer, int32 x, int32 y, uint16 tile);
    void (*CopyTileLayer)(uint16 dstLayerID, int32 dstStartX, int32 dstStartY, uint16 srcLayerID, int32 srcStartX, int32 srcStartY, int32 countX,
                           int32 countY);
    void (*ProcessParallax)(TileLayer *tileLayer);
    ScanlineInfo *(*GetScanlines)(void);

    // Object & Tile Collisions
    bool32 (*CheckObjectCollisionTouchBox)(void *thisEntity, Hitbox *thisHitbox, void *otherEntity, Hitbox *otherHitbox);
    bool32 (*CheckObjectCollisionTouchCircle)(void *thisEntity, int32 thisRadius, void *otherEntity, int32 otherRadius);
    uint8 (*CheckObjectCollisionBox)(void *thisEntity, Hitbox *thisHitbox, void *otherEntity, Hitbox *otherHitbox, bool32 setPos);
    bool32 (*CheckObjectCollisionPlatform)(void *thisEntity, Hitbox *thisHitbox, void *otherEntity, Hitbox *otherHitbox, bool32 setPos);
    bool32 (*ObjectTileCollision)(void *entity, uint16 collisionLayers, uint8 collisionMode, uint8 collisionPlane, int32 xOffset, int32 yOffset,
                                  bool32 setPos);
    bool32 (*ObjectTileGrip)(void *entity, uint16 collisionLayers, uint8 collisionMode, uint8 collisionPlane, int32 xOffset, int32 yOffset,
                             int32 tolerance);
    void (*ProcessObjectMovement)(void *entity, Hitbox *outer, Hitbox *inner);
    int32 (*GetTileAngle)(uint16 tile, uint8 cPlane, uint8 cMode);
    void (*SetTileAngle)(uint16 tile, uint8 cPlane, uint8 cMode, uint8 angle);
    uint8 (*GetTileFlags)(uint16 tile, uint8 cPlane);
    void (*SetTileFlags)(uint16 tile, uint8 cPlane, uint8 flag);

    // Audio
    uint16 (*GetSfx)(const char *path);
    int32 (*PlaySfx)(uint16 sfx, int32 loopPoint, int32 priority);
    void (*StopSfx)(uint16 sfx);
    int32 (*PlayStream)(const char *filename, uint32 channel, uint32 startPos, uint32 loopPoint, bool32 loadASync);
    void (*SetChannelAttributes)(uint8 channel, float volume, float pan, float speed);
    void (*StopChannel)(uint32 channel);
    void (*PauseChannel)(uint32 channel);
    void (*ResumeChannel)(uint32 channel);
    bool32 (*IsSfxPlaying)(uint16 sfx);
    bool32 (*ChannelActive)(uint32 channel);
    uint32 (*GetChannelPos)(uint32 channel);

    // Videos & "HD Images"
    bool32 (*LoadVideo)(const char *filename, double startDelay, bool32 (*skipCallback)(void));
    bool32 (*LoadImage)(const char *filename, double displayLength, double fadeSpeed, bool32 (*skipCallback)(void));

    // Input
    uint32 (*GetInputDeviceID)(uint8 inputSlot);
    uint32 (*GetFilteredInputDeviceID)(bool32 confirmOnly, bool32 unassignedOnly, uint32 maxInactiveTimer);
    int32 (*GetInputDeviceType)(uint32 deviceID);
    bool32 (*IsInputDeviceAssigned)(uint32 deviceID);
    int32 (*GetInputDeviceUnknown)(uint32 deviceID);
    int32 (*InputDeviceUnknown1)(uint32 deviceID, int32 unknown1, int32 unknown2);
    int32 (*InputDeviceUnknown2)(uint32 deviceID, int32 unknown1, int32 unknown2);
    int32 (*GetInputSlotUnknown)(uint8 inputSlot);
    int32 (*InputSlotUnknown1)(uint8 inputSlot, int32 unknown1, int32 unknown2);
    int32 (*InputSlotUnknown2)(uint8 inputSlot, int32 unknown1, int32 unknown2);
    void (*AssignInputSlotToDevice)(uint8 inputSlot, uint32 deviceID);
    bool32 (*IsInputSlotAssigned)(uint8 inputSlot);
    void (*ResetInputSlotAssignments)(void);

    // User File Management
    bool32 (*LoadUserFile)(const char *fileName, void *buffer, uint32 size); // load user file from exe dir
    bool32 (*SaveUserFile)(const char *fileName, void *buffer, uint32 size); // save user file to exe dir

    // Printing (Rev02)
    void (*PrintLog)(int32 mode, const char *message, ...);
    void (*PrintText)(int32 mode, const char *message);
    void (*PrintString)(int32 mode, String *message);
    void (*PrintUInt32)(int32 mode, const char *message, uint32 i);
    void (*PrintInt32)(int32 mode, const char *message, int32 i);
    void (*PrintFloat)(int32 mode, const char *message, float f);
    void (*PrintVector2)(int32 mode, const char *message, Vector2 vec);
    void (*PrintHitbox)(int32 mode, const char *message, Hitbox hitbox);

    // Editor
    void (*SetActiveVariable)(int32 classID, const char *name);
    void (*AddVarEnumValue)(const char *name);

    // Debugging
    void (*ClearViewableVariables)(void);
    void (*AddViewableVariable)(const char *name, void *value, int32 type, int32 min, int32 max);
    */
}
