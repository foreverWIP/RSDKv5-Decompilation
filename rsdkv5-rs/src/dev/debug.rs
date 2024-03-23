use crate::*;

use self::{
    engine_core::{
        engine,
        legacy::retro_engine::{legacy_gameMode, LegacyRetroStates},
    },
    graphics::drawing::{
        currentScreen, screens, Alignments, DrawDevString, DrawRectangle, InkEffects,
    },
    input::{controller, touchInfo, AssignInputSlotToDevice, InputIDs, InputSlotIDs},
    scene::{
        legacy::{
            object::Legacy_playerListPos, LegacyStageModes, Legacy_debugMode, Legacy_stageMode,
        },
        sceneInfo, EngineStates, SceneListInfo,
    },
    user::core::{SKU_CheckDLC, SKU_GetConfirmButtonFlip},
};

#[repr(i32)]
pub enum PrintModes {
    PRINT_NORMAL,
    PRINT_POPUP,
    PRINT_ERROR,
    PRINT_FATAL,
    #[cfg(feature = "version_u")]
    PRINT_SCRIPTERR,
}

#[repr(i8)]
#[derive(Eq, PartialEq)]
pub enum TouchCornerButtons {
    CORNERBUTTON_START,
    CORNERBUTTON_LEFTRIGHT,
    CORNERBUTTON_SLIDER,
}

#[repr(C)]
struct DevMenu {
    state: Option<unsafe extern "C" fn()>,
    selection: int32,
    scrollPos: int32,
    timer: int32,
    windowed: bool32,
    sceneState: int8,
    listPos: int8,
    windowScale: int8,
    windowAspect: int8,
    #[cfg(feature = "mod_loader")]
    modsChanged: bool32,
    #[cfg(feature = "mod_loader")]
    startingVersion: uint8,
    #[cfg(all(feature = "mod_loader", feature = "version_u"))]
    playerListPos: int32,
}

extern "C" {
    pub fn PrintLog(mode: PrintModes, message: *const i8, ...);

    fn DevMenu_CategorySelectMenu();
}

#[no_mangle]
static mut devMenu: DevMenu = DevMenu {
    state: None,
    selection: 0,
    scrollPos: 0,
    timer: 0,
    windowed: false32,
    sceneState: 0,
    listPos: 0,
    windowScale: 0,
    windowAspect: 0,
    modsChanged: false32,
    startingVersion: 0,
    playerListPos: 0,
};

static mut touchTimer: uint8 = 0;

#[no_mangle]
#[export_name = "DevMenu_HandleTouchControls"]
pub extern "C" fn dev_menu_handle_touch_controls(cornerButton: TouchCornerButtons) {
    unsafe {
        let cornerCheck: bool = if cornerButton != TouchCornerButtons::CORNERBUTTON_START {
            controller[InputSlotIDs::CONT_ANY as usize].keyLeft.down == true32
                && controller[InputSlotIDs::CONT_ANY as usize].keyRight.down == false32
        } else {
            controller[InputSlotIDs::CONT_ANY as usize].keyStart.down == false32
        };

        if cornerCheck
            && controller[InputSlotIDs::CONT_ANY as usize].keyUp.down == false32
            && controller[InputSlotIDs::CONT_ANY as usize].keyDown.down == false32
        {
            for t in 0..(touchInfo.count as usize) {
                let tx: int32 = (touchInfo.x[t] * screens[0].size.x as f32) as i32;
                let ty: int32 = (touchInfo.y[t] * screens[0].size.y as f32) as i32;

                let touchingSlider: bool = cornerButton == TouchCornerButtons::CORNERBUTTON_SLIDER
                    && tx > screens[0].center.x
                    && ty > screens[0].center.y;

                if (touchInfo.down[t] == true32 && (!(touchTimer % 8) != 0 || touchingSlider)) {
                    if (tx < screens[0].center.x) {
                        if (ty >= screens[0].center.y) {
                            if (controller[InputSlotIDs::CONT_ANY as usize].keyDown.down == false32)
                            {
                                controller[InputSlotIDs::CONT_ANY as usize].keyDown.press = true32;
                            }

                            controller[InputSlotIDs::CONT_ANY as usize].keyDown.down = true32;
                            break;
                        } else {
                            if (controller[InputSlotIDs::CONT_ANY as usize].keyUp.down == false32) {
                                controller[InputSlotIDs::CONT_ANY as usize].keyUp.press = true32;
                            }

                            controller[InputSlotIDs::CONT_ANY as usize].keyUp.down = true32;
                            break;
                        }
                    } else if (tx > screens[0].center.x) {
                        if (ty > screens[0].center.y) {
                            if (cornerButton == TouchCornerButtons::CORNERBUTTON_START) {
                                if (controller[InputSlotIDs::CONT_ANY as usize].keyStart.down
                                    == false32)
                                {
                                    controller[InputSlotIDs::CONT_ANY as usize].keyStart.press =
                                        true32;
                                }

                                controller[InputSlotIDs::CONT_ANY as usize].keyStart.down = true32;
                            } else {
                                if ((tx as f32) < screens[0].size.x as f32 * 0.75) {
                                    if (controller[InputSlotIDs::CONT_ANY as usize].keyLeft.down
                                        == false32)
                                    {
                                        controller[InputSlotIDs::CONT_ANY as usize].keyLeft.press =
                                            true32;
                                    }

                                    controller[InputSlotIDs::CONT_ANY as usize].keyLeft.down =
                                        true32;
                                } else {
                                    if (controller[InputSlotIDs::CONT_ANY as usize].keyRight.down
                                        == false32)
                                    {
                                        controller[InputSlotIDs::CONT_ANY as usize]
                                            .keyRight
                                            .press = true32;
                                    }

                                    controller[InputSlotIDs::CONT_ANY as usize].keyRight.down =
                                        true32;
                                    break;
                                }
                            }
                            break;
                        } else {
                            if (controller[InputSlotIDs::CONT_ANY as usize].keyB.down == false32) {
                                controller[InputSlotIDs::CONT_ANY as usize].keyB.press = true32;
                            }

                            controller[InputSlotIDs::CONT_ANY as usize].keyB.down = true32;
                            break;
                        }
                    }
                }
            }
        }

        touchTimer += 1;
    }
}

#[no_mangle]
#[export_name = "DevMenu_SceneSelectMenu"]
pub extern "C" fn dev_menu_scene_select() {
    unsafe {
        let mut selectionColors: [uint32; 8] = [
            0x808090, 0x808090, 0x808090, 0x808090, 0x808090, 0x808090, 0x808090, 0x808090,
        ];
        selectionColors[(devMenu.selection - devMenu.scrollPos) as usize] = 0xF0F0F0;

        let mut dy: int32 = (*currentScreen).center.y;
        DrawRectangle(
            (*currentScreen).center.x - 128,
            dy - 84,
            0x100,
            0x30,
            0x80,
            0xFF,
            InkEffects::INK_NONE as i32,
            true32,
        );

        dy -= 68;
        DrawDevString(
            "SELECT STAGE SCENE".as_ptr() as *const i8,
            (*currentScreen).center.x,
            dy,
            Alignments::ALIGN_CENTER as i32,
            0xF0F0F0,
        );
        DrawRectangle(
            (*currentScreen).center.x - 128,
            dy + 36,
            0x100,
            0x48,
            0x80,
            0xFF,
            InkEffects::INK_NONE as i32,
            true32,
        );

        let mut y: int32 = dy + 40;
        let list: *const SceneListInfo = sceneInfo
            .listCategory
            .wrapping_add(devMenu.listPos as usize);
        let start: int32 = (*list).sceneOffsetStart as i32;
        for i in 0..8 {
            if (devMenu.scrollPos + i < (*list).sceneCount as i32) {
                DrawDevString(
                    (*sceneInfo
                        .listData
                        .wrapping_add((start + (devMenu.scrollPos + i)) as usize))
                    .name
                    .as_ptr(),
                    (*currentScreen).center.x + 96,
                    y,
                    Alignments::ALIGN_RIGHT as i32,
                    selectionColors[i as usize],
                );
                y += 8;
                devMenu.scrollPos = devMenu.scrollPos; //? look into
            }
        }

        dev_menu_handle_touch_controls(TouchCornerButtons::CORNERBUTTON_START);

        if (controller[InputSlotIDs::CONT_ANY as usize].keyUp.press == true32) {
            if devMenu.timer == 0 {
                devMenu.selection -= 1;
                if (start + devMenu.selection < (*list).sceneOffsetStart as i32) {
                    devMenu.selection = (*list).sceneCount as i32 - 1;
                }
            }

            if (devMenu.selection >= devMenu.scrollPos) {
                if (devMenu.selection > devMenu.scrollPos + 7) {
                    devMenu.scrollPos = devMenu.selection - 7;
                }
            } else {
                devMenu.scrollPos = devMenu.selection;
            }

            devMenu.timer = 1;
        } else if (controller[InputSlotIDs::CONT_ANY as usize].keyUp.down == true32) {
            if devMenu.timer == 0 {
                devMenu.selection -= 1;
                if (start + devMenu.selection < (*list).sceneOffsetStart as i32) {
                    devMenu.selection = (*list).sceneCount as i32 - 1;
                }
            }

            devMenu.timer = (devMenu.timer + 1) & 7;

            if (devMenu.selection >= devMenu.scrollPos) {
                if (devMenu.selection > devMenu.scrollPos + 7) {
                    devMenu.scrollPos = devMenu.selection - 7;
                }
            } else {
                devMenu.scrollPos = devMenu.selection;
            }
        }

        if (controller[InputSlotIDs::CONT_ANY as usize].keyDown.press == true32) {
            devMenu.selection += 1;
            if (devMenu.selection >= (*list).sceneCount as i32) {
                devMenu.selection = 0;
            }

            if (devMenu.selection >= devMenu.scrollPos) {
                if (devMenu.selection > devMenu.scrollPos + 7) {
                    devMenu.scrollPos = devMenu.selection - 7;
                }
            } else {
                devMenu.scrollPos = devMenu.selection;
            }

            devMenu.timer = 1;
        } else if (controller[InputSlotIDs::CONT_ANY as usize].keyDown.down == true32) {
            if devMenu.timer == 0 {
                devMenu.selection += 1;
                if (devMenu.selection >= (*list).sceneCount as i32) {
                    devMenu.selection = 0;
                }
            }

            devMenu.timer = (devMenu.timer + 1) & 7;

            if (devMenu.selection >= devMenu.scrollPos) {
                if (devMenu.selection > devMenu.scrollPos + 7) {
                    devMenu.scrollPos = devMenu.selection - 7;
                }
            } else {
                devMenu.scrollPos = devMenu.selection;
            }
        }

        let mut confirm: bool32 = controller[InputSlotIDs::CONT_ANY as usize].keyA.press;
        let swap: bool32 = SKU_GetConfirmButtonFlip();
        if (swap == true32) {
            confirm = controller[InputSlotIDs::CONT_ANY as usize].keyB.press;
        }

        if (controller[InputSlotIDs::CONT_ANY as usize].keyStart.press == true32
            || confirm == true32)
        {
            let disabled: bool = if cfg!(feature = "version_2") {
                // they hardcoded a check in here that forces you to own the encore DLC to select encore mode stages
                to_string((*list).name.as_ptr()) == "Encore Mode" && SKU_CheckDLC(0) == false32
            } else {
                false
            };

            if (!disabled) {
                sceneInfo.activeCategory = devMenu.listPos as u8;
                sceneInfo.listPos = devMenu.selection as u16 + (*list).sceneOffsetStart;

                if cfg!(feature = "version_u") {
                    match engine.version {
                        5 => {
                            sceneInfo.state = EngineStates::ENGINESTATE_LOAD as u8;
                        }
                        3 | 4 => {
                            Legacy_debugMode = confirm;
                            if cfg!(feature = "mod_loader") {
                                Legacy_playerListPos = devMenu.playerListPos;
                            }
                            legacy_gameMode = LegacyRetroStates::ENGINE_MAINGAME;
                            Legacy_stageMode = LegacyStageModes::STAGEMODE_LOAD;
                        }
                        _ => {}
                    }
                } else {
                    sceneInfo.state = EngineStates::ENGINESTATE_LOAD as u8;
                }

                // Bug Details(?):
                // rev01 had this here, rev02 does not.
                // This can cause an annoying popup when starting a stage
                if !cfg!(feature = "version_2") {
                    AssignInputSlotToDevice(
                        InputSlotIDs::CONT_P1 as u8,
                        InputIDs::INPUT_AUTOASSIGN as u32,
                    );
                }
            }
        } else if (if swap == true32 {
            controller[InputSlotIDs::CONT_ANY as usize].keyA.press == true32
        } else {
            controller[InputSlotIDs::CONT_ANY as usize].keyB.press == true32
        }) {
            devMenu.state = Some(DevMenu_CategorySelectMenu);
            devMenu.scrollPos = 0;
            devMenu.selection = 0;
            devMenu.listPos = 0;
        }
    }
}
