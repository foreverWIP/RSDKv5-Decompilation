use crate::{
    graphics::drawing::ScreenInfo,
    input::{AnalogState, ControllerState, TouchInfo, TriggerState},
    scene::SceneInfo,
    storage::text::GameVersionInfo,
    user::core::{SKUInfo, UnknownInfo},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "version_2")] {
        pub type LogicLinkHandle = extern "C" fn(*const EngineInfo);

        pub struct EngineInfo {
            functionTable: *const u8,
            APITable: *const u8,

            gameInfo: *mut GameVersionInfo,
            currentSKU: *mut SKUInfo,
            sceneInfo: *mut SceneInfo,

            controller: *mut ControllerState,
            stickL: *mut AnalogState,
            stickR: *mut AnalogState,
            triggerL: *mut TriggerState,
            triggerR: *mut TriggerState,
            touchMouse: *mut TouchInfo,

            unknown: *mut UnknownInfo,

            screenInfo: *mut ScreenInfo,

            #[cfg(feature = "version_u")]
            // only for origins, not technically needed for v5U if standalone I think
            hedgehogLink: *mut u8,

            #[cfg(feature = "mod_loader")]
            modTable: *mut u8,
        }
    } else {
        pub type LogicLinkHandle = extern "C" fn(info: EngineInfo);

        pub struct EngineInfo {
            functionTable: *mut u8,

            gameInfo: *mut GameVersionInfo,
            sceneInfo: *mut SceneInfo,

            controllerInfo: *mut ControllerState,
            stickInfo: *mut AnalogState,

            touchInfo: *mut TouchInfo,

            screenInfo: *mut ScreenInfo,

            #[cfg(feature = "mod_loader")]
            modTable: *mut u8,
        }
    }
}

#[no_mangle]
pub static mut linkGameLogic: Option<LogicLinkHandle> = None;
