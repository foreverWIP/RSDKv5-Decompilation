use crate::*;

#[no_mangle]
static mut legacy_gameMode: int32 = 0;
#[no_mangle]
static mut legacy_usingBytecode: bool32 = bool32::False;

#[no_mangle]
static mut legacy_trialMode: bool32 = bool32::False;
#[no_mangle]
static mut legacy_gamePlatformID: int32 = 0;
#[no_mangle]
static mut legacy_deviceType: int32 = 0;
#[no_mangle]
static mut legacy_onlineActive: bool32 = bool32::False;
#[no_mangle]
static mut legacy_language: int32 = 0;
#[cfg(feature = "legacy_use_haptics")]
#[no_mangle]
static mut legacy_hapticsEnabled: bool32 = bool32::False;

#[no_mangle]
static mut sinM7LookupTable: [int32; 0x200] = [0; 0x200];
#[no_mangle]
static mut cosM7LookupTable: [int32; 0x200] = [0; 0x200];
