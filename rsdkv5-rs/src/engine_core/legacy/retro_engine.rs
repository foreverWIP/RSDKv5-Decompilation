use crate::*;

use self::engine_core::math::RSDK_PI;

#[repr(C)]
enum LegacyRetroStates {
    ENGINE_DEVMENU = 0,
    ENGINE_MAINGAME = 1,
    ENGINE_INITDEVMENU = 2,

    ENGINE_SCRIPTERROR = 4,
}

#[no_mangle]
static mut legacy_gameMode: int32 = LegacyRetroStates::ENGINE_MAINGAME as i32;
#[no_mangle]
static mut legacy_usingBytecode: bool32 = false32;

#[no_mangle]
static mut legacy_trialMode: bool32 = false32;
#[no_mangle]
static mut legacy_gamePlatformID: int32 = 0;
#[no_mangle]
static mut legacy_deviceType: int32 = 0;
#[no_mangle]
static mut legacy_onlineActive: bool32 = false32;
#[no_mangle]
static mut legacy_language: int32 = 0;
#[cfg(feature = "legacy_use_haptics")]
#[no_mangle]
static mut legacy_hapticsEnabled: bool32 = false32;

#[no_mangle]
static mut sinM7LookupTable: [int32; 0x200] = [0; 0x200];
#[no_mangle]
static mut cosM7LookupTable: [int32; 0x200] = [0; 0x200];

#[no_mangle]
#[export_name = "CalculateTrigAnglesM7"]
pub extern "C" fn calc_trig_angles_m7() {
    unsafe {
        for i in 0..0x200 {
            sinM7LookupTable[i] = (f32::sin(((i as f32) / 256.0) * RSDK_PI) * 4096.0) as int32;
            cosM7LookupTable[i] = (f32::cos(((i as f32) / 256.0) * RSDK_PI) * 4096.0) as int32;
        }

        cosM7LookupTable[0x00] = 0x1000;
        cosM7LookupTable[0x80] = 0;
        cosM7LookupTable[0x100] = -0x1000;
        cosM7LookupTable[0x180] = 0;

        sinM7LookupTable[0x00] = 0;
        sinM7LookupTable[0x80] = 0x1000;
        sinM7LookupTable[0x100] = 0;
        sinM7LookupTable[0x180] = -0x1000;
    }
}
