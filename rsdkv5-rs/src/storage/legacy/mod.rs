use crate::*;

use self::{
    dev::debug::{PrintLog, PrintModes},
    engine_core::legacy::retro_engine::{legacy_gameMode, LegacyRetroStates},
    scene::{
        legacy::{LegacyStageModes, Legacy_stageMode},
        sceneInfo,
    },
};

use super::text::str_comp;

const LEGACY_GLOBALVAR_COUNT: usize = 0x100;

const LEGACY_SAVEDATA_SIZE: usize = 0x2000;

cfg_if::cfg_if! {
    if #[cfg(feature = "mod_loader")] {
        const LEGACY_v4_NATIIVEFUNCTION_COUNT: usize = 0x30;
    } else {
        const LEGACY_v4_NATIIVEFUNCTION_COUNT: usize = 0x10;
    }
}

#[repr(C)]
struct GlobalVariable {
    name: [i8; 0x20],
    value: i32,
}
const DEFAULT_GLOBALVARIABLE: GlobalVariable = GlobalVariable {
    name: [0; 0x20],
    value: 0,
};

#[no_mangle]
static mut Legacy_nativeFunction: [*const u8; LEGACY_v4_NATIIVEFUNCTION_COUNT] =
    [std::ptr::null(); LEGACY_v4_NATIIVEFUNCTION_COUNT];
#[no_mangle]
static mut Legacy_nativeFunctionCount: int32 = 0;

#[no_mangle]
static mut Legacy_globalVariablesCount: int32 = 0;
#[no_mangle]
static mut Legacy_globalVariables: [GlobalVariable; LEGACY_GLOBALVAR_COUNT] =
    [DEFAULT_GLOBALVARIABLE; LEGACY_GLOBALVAR_COUNT];

#[no_mangle]
static mut Legacy_saveRAM: [int32; LEGACY_SAVEDATA_SIZE] = [0; LEGACY_SAVEDATA_SIZE];

#[repr(i32)]
enum NotifyCallbackIDs {
    NOTIFY_DEATH_EVENT = 128,
    NOTIFY_TOUCH_SIGNPOST = 129,
    NOTIFY_HUD_ENABLE = 130,
    NOTIFY_ADD_COIN = 131,
    NOTIFY_KILL_ENEMY = 132,
    NOTIFY_SAVESLOT_SELECT = 133,
    NOTIFY_FUTURE_PAST = 134,
    NOTIFY_GOTO_FUTURE_PAST = 135,
    NOTIFY_BOSS_END = 136,
    NOTIFY_SPECIAL_END = 137,
    NOTIFY_DEBUGPRINT = 138,
    NOTIFY_KILL_BOSS = 139,
    NOTIFY_TOUCH_EMERALD = 140,
    NOTIFY_STATS_ENEMY = 141,
    NOTIFY_STATS_CHARA_ACTION = 142,
    NOTIFY_STATS_RING = 143,
    NOTIFY_STATS_MOVIE = 144,
    NOTIFY_STATS_PARAM_1 = 145,
    NOTIFY_STATS_PARAM_2 = 146,
    NOTIFY_CHARACTER_SELECT = 147,
    NOTIFY_SPECIAL_RETRY = 148,
    NOTIFY_TOUCH_CHECKPOINT = 149,
    NOTIFY_ACT_FINISH = 150,
    NOTIFY_1P_VS_SELECT = 151,
    NOTIFY_CONTROLLER_SUPPORT = 152,
    NOTIFY_STAGE_RETRY = 153,
    NOTIFY_SOUND_TRACK = 154,
    NOTIFY_GOOD_ENDING = 155,
    NOTIFY_BACK_TO_MAINMENU = 156,
    NOTIFY_LEVEL_SELECT_MENU = 157,
    NOTIFY_PLAYER_SET = 158,
    NOTIFY_EXTRAS_MODE = 159,
    NOTIFY_SPIN_DASH_TYPE = 160,
    NOTIFY_TIME_OVER = 161,
    NOTIFY_TIMEATTACK_MODE = 162,
    NOTIFY_STATS_BREAK_OBJECT = 163,
    NOTIFY_STATS_SAVE_FUTURE = 164,
    NOTIFY_STATS_CHARA_ACTION2 = 165,
}

#[no_mangle]
#[export_name = "Legacy_GetGlobalVariableByName"]
pub extern "C" fn legacy_get_global_variable_by_name_ptr(name: *const i8) -> int32 {
    unsafe {
        for v in 0..(Legacy_globalVariablesCount as usize) {
            if (str_comp(name, Legacy_globalVariables[v].name.as_ptr())) {
                return Legacy_globalVariables[v].value;
            }
        }
    }
    return 0;
}

pub fn legacy_get_global_variable_by_name(name: &str) -> int32 {
    legacy_get_global_variable_by_name_ptr(name.as_ptr() as *const i8)
}

#[no_mangle]
#[export_name = "Legacy_SetGlobalVariableByName"]
pub extern "C" fn legacy_set_global_variable_by_name_ptr(name: *const i8, value: i32) {
    unsafe {
        for v in 0..(Legacy_globalVariablesCount as usize) {
            if (str_comp(name, Legacy_globalVariables[v].name.as_ptr())) {
                Legacy_globalVariables[v].value = value;
                break;
            }
        }
    }
}
pub fn legacy_set_global_variable_by_name(name: &str, value: i32) {
    legacy_set_global_variable_by_name_ptr(name.as_ptr() as *const i8, value);
}

#[no_mangle]
#[export_name = "Legacy_GetGlobalVariableID"]
pub extern "C" fn legacy_get_global_variable_id_ptr(name: *const i8) -> int32 {
    unsafe {
        for v in 0..(Legacy_globalVariablesCount as usize) {
            if (str_comp(name, Legacy_globalVariables[v].name.as_ptr())) {
                return v as i32;
            }
        }
    }
    return 0xFF;
}

pub fn legacy_get_global_variable_id(name: &str) -> int32 {
    legacy_get_global_variable_id_ptr(name.as_ptr() as *const i8)
}

#[no_mangle]
#[export_name = "HapticEffect"]
pub extern "C" fn haptic_effect(
    id: *const i32,
    unknown1: *const i32,
    unknown2: *const i32,
    unknown3: *const i32,
) {
}

#[no_mangle]
#[export_name = "NotifyCallback"]
pub extern "C" fn v4_notify_callback(
    callback: *const NotifyCallbackIDs,
    param1: *const int32,
    param2: *const int32,
    param3: *const int32,
) {
    if (callback.is_null() || param1.is_null()) {
        return;
    }

    unsafe {
        match *callback {
            NotifyCallbackIDs::NOTIFY_DEATH_EVENT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: DeathEvent() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_TOUCH_SIGNPOST => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: TouchSignPost() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_HUD_ENABLE => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: HUDEnable() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_ADD_COIN => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: AddCoin() -> %d".as_ptr() as *const i8,
                    *param1,
                );
                legacy_set_global_variable_by_name(
                    "game.coinCount",
                    legacy_get_global_variable_by_name("game.coinCount") + *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_KILL_ENEMY => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: KillEnemy() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_SAVESLOT_SELECT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: SaveSlotSelect() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_FUTURE_PAST => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: FuturePast() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_GOTO_FUTURE_PAST => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: GotoFuturePast() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_BOSS_END => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: BossEnd() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_SPECIAL_END => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: SpecialEnd() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_DEBUGPRINT => {
                // Although there are instances of this being called from both CallNativeFunction2 and CallNativeFunction4 in Origins' scripts, there's no way we can tell which one was used here to handle possible errors
                // Due to this, we'll only print param1 regardless of the opcode used
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: DebugPrint() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_KILL_BOSS => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: KillBoss() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_TOUCH_EMERALD => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: TouchEmerald() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_ENEMY => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsEnemy() -> %d, %d, %d".as_ptr() as *const i8,
                    *param1,
                    *param2,
                    *param3,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_CHARA_ACTION => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsCharaAction() -> %d, %d, %d".as_ptr() as *const i8,
                    *param1,
                    *param2,
                    *param3,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_RING => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsRing() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_MOVIE => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsMovie() -> %d".as_ptr() as *const i8,
                    *param1,
                );
                sceneInfo.activeCategory = 0;
                sceneInfo.listPos = 0;
                legacy_gameMode = LegacyRetroStates::ENGINE_MAINGAME;
                Legacy_stageMode = LegacyStageModes::STAGEMODE_LOAD;
            }
            NotifyCallbackIDs::NOTIFY_STATS_PARAM_1 => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsParam1() -> %d, %d, %d".as_ptr() as *const i8,
                    *param1,
                    *param2,
                    *param3,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_PARAM_2 => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsParam2() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_CHARACTER_SELECT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: CharacterSelect() -> %d".as_ptr() as *const i8,
                    *param1,
                );
                legacy_set_global_variable_by_name("game.callbackResult", 1);
                legacy_set_global_variable_by_name("game.continueFlag", 0);
            }
            NotifyCallbackIDs::NOTIFY_SPECIAL_RETRY => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: SpecialRetry() -> %d, %d, %d".as_ptr() as *const i8,
                    *param1,
                    *param2,
                    *param3,
                );
                legacy_set_global_variable_by_name("game.callbackResult", 1);
            }
            NotifyCallbackIDs::NOTIFY_TOUCH_CHECKPOINT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: TouchCheckpoint() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_ACT_FINISH => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: ActFinish() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_1P_VS_SELECT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: 1PVSSelect() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_CONTROLLER_SUPPORT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: ControllerSupport() -> %d".as_ptr() as *const i8,
                    *param1,
                );
                legacy_set_global_variable_by_name("game.callbackResult", 1);
            }
            NotifyCallbackIDs::NOTIFY_STAGE_RETRY => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StageRetry() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_SOUND_TRACK => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: SoundTrack() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_GOOD_ENDING => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: GoodEnding() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_BACK_TO_MAINMENU => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: BackToMainMenu() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_LEVEL_SELECT_MENU => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: LevelSelectMenu() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_PLAYER_SET => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: PlayerSet() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_EXTRAS_MODE => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: ExtrasMode() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_SPIN_DASH_TYPE => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: SpindashType() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_TIME_OVER => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: TimeOver() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_TIMEATTACK_MODE => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: TimeAttackMode() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_BREAK_OBJECT => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsBreakObject() -> %d, %d".as_ptr() as *const i8,
                    *param1,
                    *param2,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_SAVE_FUTURE => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsSaveFuture() -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
            NotifyCallbackIDs::NOTIFY_STATS_CHARA_ACTION2 => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: StatsCharaAction2() -> %d, %d, %d".as_ptr() as *const i8,
                    *param1,
                    *param2,
                    *param3,
                );
            }
            _ => {
                PrintLog(
                    PrintModes::PRINT_NORMAL,
                    "NOTIFY: Unknown Callback -> %d".as_ptr() as *const i8,
                    *param1,
                );
            }
        }
    }
}
