use crate::*;

use dlopen::raw::Library;

use crate::{
    graphics::drawing::ScreenInfo,
    input::{AnalogState, ControllerState, TouchInfo, TriggerState},
    scene::SceneInfo,
    storage::text::GameVersionInfo,
    user::core::{SKUInfo, UnknownInfo},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle {
    handle: *const Library,
}

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

static mut game_library: Option<Library> = None;

static mut error_string: String = String::new();

#[no_mangle]
#[export_name = "Link_Open"]
pub extern "C" fn link_open(path: *const i8) -> Handle {
    unsafe {
        let path = to_string(path)
            + if cfg!(target_os = "windows") {
                ".dll"
            } else if cfg!(target_family = "macos") {
                ".dylib"
            } else {
                ".so"
            };

        game_library = match Library::open(path) {
            Ok(l) => Some(l),
            Err(e) => {
                error_string = e.to_string();
                None
            }
        };
        Handle {
            handle: match &game_library {
                Some(l) => l,
                None => std::ptr::null(),
            },
        }
    }
}

#[no_mangle]
#[export_name = "Link_Close"]
pub extern "C" fn link_close(_handle: Handle) {}

#[no_mangle]
#[export_name = "Link_GetSymbol"]
pub extern "C" fn link_get_symbol(handle: Handle, symbol: *const i8) -> *const u8 {
    if handle.handle.is_null() {
        return std::ptr::null();
    }

    unsafe {
        match (*handle.handle).symbol(&to_string(symbol)) {
            Ok(s) => s,
            Err(e) => {
                error_string = e.to_string();
                std::ptr::null()
            }
        }
    }
}

#[no_mangle]
#[export_name = "Link_GetError"]
pub extern "C" fn get_error() -> *const i8 {
    unsafe { error_string.as_ptr() as *const i8 }
}
