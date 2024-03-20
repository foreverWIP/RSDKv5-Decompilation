use crate::*;

const PLAYER_COUNT: usize = 4;
const INPUTDEVICE_COUNT: usize = 0x10;

const INPUT_DEADZONE: f64 = 0.3;

#[repr(C)]
pub enum InputIDs {
    INPUT_UNASSIGNED = -2,
    INPUT_AUTOASSIGN = -1,
    INPUT_NONE = 0,
}

#[repr(C)]
pub enum InputSlotIDs {
    CONT_ANY,
    CONT_P1,
    CONT_P2,
    CONT_P3,
    CONT_P4,
}

#[repr(C)]
pub enum InputDeviceTypes {
    DEVICE_TYPE_NONE,
    DEVICE_TYPE_KEYBOARD,
    DEVICE_TYPE_CONTROLLER,
    DEVICE_TYPE_UNKNOWN,
    DEVICE_TYPE_STEAMOVERLAY,
}

#[repr(C)]
pub enum InputDeviceIDs {
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
pub struct GamePadButtonMap {
    maskVal: int32,
    mappingType: int16,
    offset: int16,
}

#[repr(C)]
pub struct GamePadMappings {
    name: [i8; 0x40],
    buttons: [GamePadButtonMap; 24],
    vendorID: int32,
    productID: int32,
    type_: int32,
}

#[repr(C)]
pub struct InputDevice {
    gamepadType: int32,
    id: uint32,
    active: uint8,
    isAssigned: uint8,
    unused: uint8,
    disabled: uint8,
    anyPress: uint8,
    inactiveTimer: [int32; 2],
}

#[repr(C)]
pub struct InputState {
    down: bool32,
    press: bool32,
    keyMap: int32,
}
const DEFAULT_INPUTSTATE: InputState = InputState {
    down: false32,
    press: false32,
    keyMap: 0,
};

#[repr(C)]
pub struct ControllerState {
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

    // Rev01 hasn't split these into different structs yet
    #[cfg(not(feature = "version_2"))]
    keyBumperL: InputState,
    #[cfg(not(feature = "version_2"))]
    keyBumperR: InputState,
    #[cfg(not(feature = "version_2"))]
    keyTriggerL: InputState,
    #[cfg(not(feature = "version_2"))]
    keyTriggerR: InputState,
    #[cfg(not(feature = "version_2"))]
    keyStickL: InputState,
    #[cfg(not(feature = "version_2"))]
    keyStickR: InputState,
}
const DEFAULT_CONTROLLERSTATE: ControllerState = ControllerState {
    keyUp: DEFAULT_INPUTSTATE,
    keyDown: DEFAULT_INPUTSTATE,
    keyLeft: DEFAULT_INPUTSTATE,
    keyRight: DEFAULT_INPUTSTATE,
    keyA: DEFAULT_INPUTSTATE,
    keyB: DEFAULT_INPUTSTATE,
    keyC: DEFAULT_INPUTSTATE,
    keyX: DEFAULT_INPUTSTATE,
    keyY: DEFAULT_INPUTSTATE,
    keyZ: DEFAULT_INPUTSTATE,
    keyStart: DEFAULT_INPUTSTATE,
    keySelect: DEFAULT_INPUTSTATE,

    // Rev01 hasn't split these into different structs yet
    #[cfg(not(feature = "version_2"))]
    keyBumperL: DEFAULT_INPUTSTATE,
    #[cfg(not(feature = "version_2"))]
    keyBumperR: DEFAULT_INPUTSTATE,
    #[cfg(not(feature = "version_2"))]
    keyTriggerL: DEFAULT_INPUTSTATE,
    #[cfg(not(feature = "version_2"))]
    keyTriggerR: DEFAULT_INPUTSTATE,
    #[cfg(not(feature = "version_2"))]
    keyStickL: DEFAULT_INPUTSTATE,
    #[cfg(not(feature = "version_2"))]
    keyStickR: DEFAULT_INPUTSTATE,
};

#[repr(C)]
pub struct AnalogState {
    keyUp: InputState,
    keyDown: InputState,
    keyLeft: InputState,
    keyRight: InputState,
    #[cfg(feature = "version_2")]
    keyStick: InputState,
    #[cfg(feature = "version_2")]
    deadzone: float,
    #[cfg(feature = "version_2")]
    hDelta: float,
    #[cfg(feature = "version_2")]
    vDelta: float,
    #[cfg(not(feature = "version_2"))]
    deadzone: float,
    #[cfg(not(feature = "version_2"))]
    triggerDeltaL: float,
    #[cfg(not(feature = "version_2"))]
    triggerDeltaR: float,
    #[cfg(not(feature = "version_2"))]
    hDeltaL: float,
    #[cfg(not(feature = "version_2"))]
    vDeltaL: float,
    #[cfg(not(feature = "version_2"))]
    hDeltaR: float,
    #[cfg(not(feature = "version_2"))]
    vDeltaR: float,
}
const DEFAULT_ANALOGSTATE: AnalogState = AnalogState {
    keyUp: DEFAULT_INPUTSTATE,
    keyDown: DEFAULT_INPUTSTATE,
    keyLeft: DEFAULT_INPUTSTATE,
    keyRight: DEFAULT_INPUTSTATE,
    #[cfg(feature = "version_2")]
    keyStick: DEFAULT_INPUTSTATE,
    #[cfg(feature = "version_2")]
    deadzone: 0.0,
    #[cfg(feature = "version_2")]
    hDelta: 0.0,
    #[cfg(feature = "version_2")]
    vDelta: 0.0,
    #[cfg(not(feature = "version_2"))]
    deadzone: 0.0,
    #[cfg(not(feature = "version_2"))]
    triggerDeltaL: 0.0,
    #[cfg(not(feature = "version_2"))]
    triggerDeltaR: 0.0,
    #[cfg(not(feature = "version_2"))]
    hDeltaL: 0.0,
    #[cfg(not(feature = "version_2"))]
    vDeltaL: 0.0,
    #[cfg(not(feature = "version_2"))]
    hDeltaR: 0.0,
    #[cfg(not(feature = "version_2"))]
    vDeltaR: 0.0,
};

#[cfg(feature = "version_2")]
#[repr(C)]
pub struct TriggerState {
    keyBumper: InputState,
    keyTrigger: InputState,
    bumperDelta: float,
    triggerDelta: float,
}
const DEFAULT_TRIGGERSTATE: TriggerState = TriggerState {
    keyBumper: DEFAULT_INPUTSTATE,
    keyTrigger: DEFAULT_INPUTSTATE,
    bumperDelta: 0.0,
    triggerDelta: 0.0,
};

struct TouchInfo {
    x: [float; 0x10],
    y: [float; 0x10],
    down: [bool32; 0x10],
    count: uint8,
    #[cfg(not(feature = "version_2"))]
    pauseHold: bool32,
    #[cfg(not(feature = "version_2"))]
    pausePress: bool32,
    #[cfg(not(feature = "version_2"))]
    unknown1: bool32,
    #[cfg(not(feature = "version_2"))]
    anyKeyHold: bool32,
    #[cfg(not(feature = "version_2"))]
    anyKeyPress: bool32,
    #[cfg(not(feature = "version_2"))]
    unknown2: bool32,
}
const DEFAULT_TOUCHINFO: TouchInfo = TouchInfo {
    x: [0.0; 0x10],
    y: [0.0; 0x10],
    down: [false32; 0x10],
    count: 0,
    #[cfg(not(feature = "version_2"))]
    pauseHold: false32,
    #[cfg(not(feature = "version_2"))]
    pausePress: false32,
    #[cfg(not(feature = "version_2"))]
    unknown1: false32,
    #[cfg(not(feature = "version_2"))]
    anyKeyHold: false32,
    #[cfg(not(feature = "version_2"))]
    anyKeyPress: false32,
    #[cfg(not(feature = "version_2"))]
    unknown2: false32,
};

#[no_mangle]
static mut inputDeviceList: [*mut InputDevice; INPUTDEVICE_COUNT] =
    [std::ptr::null_mut(); INPUTDEVICE_COUNT];
#[no_mangle]
static mut inputDeviceCount: int32 = 0;

#[no_mangle]
static mut inputSlots: [i32; PLAYER_COUNT] = [
    InputIDs::INPUT_NONE as i32,
    InputIDs::INPUT_NONE as i32,
    InputIDs::INPUT_NONE as i32,
    InputIDs::INPUT_NONE as i32,
];
#[no_mangle]
static mut inputSlotDevices: [*mut InputDevice; PLAYER_COUNT] = [
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
];

#[no_mangle]
static mut controller: [ControllerState; PLAYER_COUNT + 1] =
    [DEFAULT_CONTROLLERSTATE; PLAYER_COUNT + 1];
#[no_mangle]
static mut stickL: [AnalogState; PLAYER_COUNT + 1] = [DEFAULT_ANALOGSTATE; PLAYER_COUNT + 1];
#[cfg(feature = "version_2")]
#[no_mangle]
static mut stickR: [AnalogState; PLAYER_COUNT + 1] = [DEFAULT_ANALOGSTATE; PLAYER_COUNT + 1];
#[cfg(feature = "version_2")]
#[no_mangle]
static mut triggerL: [TriggerState; PLAYER_COUNT + 1] = [DEFAULT_TRIGGERSTATE; PLAYER_COUNT + 1];
#[cfg(feature = "version_2")]
#[no_mangle]
static mut triggerR: [TriggerState; PLAYER_COUNT + 1] = [DEFAULT_TRIGGERSTATE; PLAYER_COUNT + 1];
#[no_mangle]
static mut touchInfo: TouchInfo = DEFAULT_TOUCHINFO;

#[no_mangle]
static mut gamePadMappings: *mut GamePadMappings = std::ptr::null_mut();
#[no_mangle]
static mut gamePadCount: int32 = 0;
