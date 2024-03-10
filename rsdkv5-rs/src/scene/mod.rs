pub mod collision;
pub mod legacy;

use crate::*;

use self::{
    graphics::{
        drawing::{currentScreen, CAMERA_COUNT, LAYER_COUNT},
        palette::{fullPalette, gfxLineBuffer},
    },
    storage::text::HashMD5,
};
const TILE_COUNT: usize = 0x400;
const TILE_SIZE: usize = 0x10;
const TILE_DATASIZE: usize = TILE_SIZE * TILE_SIZE;
const TILESET_SIZE: usize = TILE_COUNT * TILE_DATASIZE;

const CPATH_COUNT: usize = 2;

const RSDK_SIGNATURE_CFG: u32 = 0x474643; // "CFG"
const RSDK_SIGNATURE_SCN: u32 = 0x4E4353; // "SCN"
const RSDK_SIGNATURE_TIL: u32 = 0x4C4954; // "TIL"

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ScrollInfo {
    tilePos: int32,
    parallaxFactor: int32,
    scrollSpeed: int32,
    scrollPos: int32,
    deform: uint8,
    unknown: uint8, // stored in the scene, but always 0, never referenced in-engine either...
}
impl ScrollInfo {
    pub const fn new() -> Self {
        Self {
            tilePos: 0,
            parallaxFactor: 0,
            scrollSpeed: 0,
            scrollPos: 0,
            deform: 0,
            unknown: 0,
        }
    }
}

#[repr(C)]
pub struct ScanlineInfo {
    position: Vector2, // position of the scanline
    deform: Vector2,   // deformation that should be applied (only applies to RotoZoom type)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TileLayer {
    type_: uint8,
    drawGroup: [uint8; CAMERA_COUNT],
    widthShift: uint8,
    heightShift: uint8,
    xsize: uint16,
    ysize: uint16,
    position: Vector2,
    parallaxFactor: int32,
    scrollSpeed: int32,
    scrollPos: int32,
    deformationOffset: int32,
    deformationOffsetW: int32,
    deformationData: [int32; 0x400],
    deformationDataW: [int32; 0x400],
    scanlineCallback: extern "C" fn(*mut ScanlineInfo),
    scrollInfoCount: uint16,
    scrollInfo: [ScrollInfo; 0x100],
    name: HashMD5,
    layout: *mut uint16,
    lineScroll: *mut uint8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CollisionMask {
    floorMasks: [uint8; TILE_SIZE],
    lWallMasks: [uint8; TILE_SIZE],
    rWallMasks: [uint8; TILE_SIZE],
    roofMasks: [uint8; TILE_SIZE],
}
impl CollisionMask {
    pub const fn new() -> Self {
        Self {
            floorMasks: [0; TILE_SIZE],
            lWallMasks: [0; TILE_SIZE],
            rWallMasks: [0; TILE_SIZE],
            roofMasks: [0; TILE_SIZE],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TileInfo {
    floorAngle: uint8,
    lWallAngle: uint8,
    rWallAngle: uint8,
    roofAngle: uint8,
    flag: uint8,
}
impl TileInfo {
    pub const fn new() -> Self {
        Self {
            floorAngle: 0,
            lWallAngle: 0,
            rWallAngle: 0,
            roofAngle: 0,
            flag: 0,
        }
    }
}

#[repr(C)]
pub struct SceneListInfo {
    hash: HashMD5,
    name: [i8; 0x20],
    sceneOffsetStart: uint16,
    sceneOffsetEnd: uint16,
    sceneCount: uint8,
}

#[repr(C)]
pub struct SceneListEntry {
    hash: HashMD5,
    name: [i8; 0x20],
    folder: [i8; 0x10],
    id: [i8; 0x08],
    #[cfg(feature = "version_2")]
    filter: u8,
}

#[repr(C)]
pub struct SceneInfo {
    entity: *mut Entity,
    listData: *mut SceneListEntry,
    listCategory: *mut SceneListInfo,
    timeCounter: int32,
    currentDrawGroup: int32,
    currentScreenID: int32,
    pub listPos: uint16,
    entitySlot: uint16,
    createSlot: uint16,
    classCount: uint16,
    inEditor: bool32,
    effectGizmo: bool32,
    debugMode: bool32,
    useGlobalObjects: bool32,
    timeEnabled: bool32,
    pub activeCategory: uint8,
    categoryCount: uint8,
    state: uint8,
    #[cfg(feature = "version_2")]
    filter: uint8,
    milliseconds: uint8,
    seconds: uint8,
    minutes: uint8,
}
impl SceneInfo {
    pub const fn new() -> Self {
        Self {
            entity: std::ptr::null_mut(),
            listData: std::ptr::null_mut(),
            listCategory: std::ptr::null_mut(),
            timeCounter: 0,
            currentDrawGroup: 0,
            currentScreenID: 0,
            listPos: 0,
            entitySlot: 0,
            createSlot: 0,
            classCount: 0,
            inEditor: false32,
            effectGizmo: false32,
            debugMode: false32,
            useGlobalObjects: false32,
            timeEnabled: false32,
            activeCategory: 0,
            categoryCount: 0,
            state: 0,
            filter: 0,
            milliseconds: 0,
            seconds: 0,
            minutes: 0,
        }
    }
}

extern "C" fn default_scanline_callback(_scanline_info: *mut ScanlineInfo) {}

#[no_mangle]
pub static mut scanlines: *mut ScanlineInfo = std::ptr::null_mut();
#[no_mangle]
pub static mut tilesetPixels: [uint8; TILESET_SIZE * 4] = [0; TILESET_SIZE * 4];

#[no_mangle]
pub static mut tileLayers: [TileLayer; LAYER_COUNT] = [TileLayer {
    type_: 0,
    drawGroup: [0; 4],
    widthShift: 0,
    heightShift: 0,
    xsize: 0,
    ysize: 0,
    position: Vector2::new(),
    parallaxFactor: 0,
    scrollSpeed: 0,
    scrollPos: 0,
    deformationOffset: 0,
    deformationOffsetW: 0,
    deformationData: [0; 1024],
    deformationDataW: [0; 1024],
    scanlineCallback: default_scanline_callback,
    scrollInfoCount: 0,
    scrollInfo: [ScrollInfo::new(); 256],
    name: [0; 4],
    layout: std::ptr::null_mut(),
    lineScroll: std::ptr::null_mut(),
}; LAYER_COUNT];
#[no_mangle]
pub static mut collisionMasks: [[CollisionMask; TILE_COUNT * 4]; CPATH_COUNT] =
    [[CollisionMask::new(); TILE_COUNT * 4]; CPATH_COUNT];
#[no_mangle]
pub static mut tileInfo: [[TileInfo; TILE_COUNT * 4]; CPATH_COUNT] =
    [[TileInfo::new(); TILE_COUNT * 4]; CPATH_COUNT];

#[cfg(feature = "version_2")]
#[no_mangle]
pub static mut forceHardReset: bool32 = false32;
#[no_mangle]
pub static mut currentSceneFolder: [i8; 0x10] = [0; 0x10];
#[no_mangle]
pub static mut currentSceneID: [i8; 0x10] = [0; 0x10];
#[no_mangle]
pub static mut sceneInfo: SceneInfo = SceneInfo::new();

#[no_mangle]
#[export_name = "DrawLayerHScroll"]
pub extern "C" fn draw_layer_hscroll(layer: &TileLayer) {
    unsafe {
        if layer.xsize == 0 || layer.ysize == 0 {
            return;
        }

        let lineTileCount = ((*currentScreen).pitch >> 4) - 1;
        let mut frameBuffer: *mut uint16 = &mut (*currentScreen).frameBuffer
            [((*currentScreen).pitch * (*currentScreen).clipBound_Y1) as usize];

        for draw_y in (*currentScreen).clipBound_Y1..(*currentScreen).clipBound_Y2 {
            let scanline = scanlines.wrapping_add(draw_y as usize).as_ref().unwrap();
            let mut x = scanline.position.x;
            let y = scanline.position.y;
            let tileX = FROM_FIXED!(x);
            let lineBuffer = gfxLineBuffer[draw_y as usize];
            let activePalette = &fullPalette[lineBuffer as usize];

            if (tileX as usize) >= TILE_SIZE * (layer.xsize as usize) {
                x = TO_FIXED!(tileX - TILE_SIZE as i32 * layer.xsize as i32);
            } else if tileX < 0 {
                x = TO_FIXED!(tileX + TILE_SIZE as i32 * layer.xsize as i32);
            }

            let mut tileRemain = TILE_SIZE as i32 - (FROM_FIXED!(x) & 0xF);
            let sheetX = FROM_FIXED!(x) & 0xF;
            let sheetY = TILE_SIZE as i32 * (FROM_FIXED!(y) & 0xF);
            let mut lineRemain = (*currentScreen).pitch;

            let mut tx = x >> 20;
            let mut layout: *const u16 = layer
                .layout
                .wrapping_add((tx + ((y >> 20) << layer.widthShift)) as usize);
            lineRemain -= tileRemain;

            if *layout < 0xFFFF {
                let pixels = &tilesetPixels
                    [TILE_DATASIZE * ((*layout & 0xFFF) as usize) + (sheetY + sheetX) as usize..];
                for p in 0..tileRemain {
                    let pixel = pixels[p as usize];
                    if pixel != 0 {
                        *frameBuffer.wrapping_add(p as usize) = activePalette[pixel as usize];
                    }
                }
            }
            frameBuffer = frameBuffer.wrapping_add(tileRemain as usize);

            for _ in 0..lineTileCount {
                layout = layout.wrapping_add(1);

                tx += 1;
                if tx == layer.xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub(layer.xsize as usize);
                }

                if *layout < 0xFFFF {
                    let pixels = &tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetY as usize..];

                    for p in 0..16 {
                        let index = pixels[p];
                        if index != 0 {
                            *frameBuffer.wrapping_add(p) = activePalette[index as usize];
                        }
                    }
                }

                frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                lineRemain = lineRemain.wrapping_sub(TILE_SIZE as i32);
            }

            while lineRemain > 0 {
                layout = layout.wrapping_add(1);

                tx += 1;
                if tx == layer.xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub(layer.xsize as usize);
                }

                tileRemain = if lineRemain >= TILE_SIZE as i32 {
                    TILE_SIZE as i32
                } else {
                    lineRemain
                };

                if *layout >= 0xFFFF {
                    frameBuffer = frameBuffer.wrapping_add(tileRemain as usize);
                } else {
                    let pixels = &tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetY as usize..];
                    for p in 0..tileRemain {
                        let index = pixels[p as usize];
                        if index != 0 {
                            *frameBuffer = activePalette[index as usize];
                        }
                        frameBuffer = frameBuffer.wrapping_add(1);
                    }
                }

                lineRemain = lineRemain.wrapping_sub(TILE_SIZE as i32);
            }
        }
    }
}

#[no_mangle]
#[export_name = "DrawLayerBasic"]
pub extern "C" fn draw_layer_basic(layer: &TileLayer) {
    unsafe {
        if layer.xsize == 0 || layer.ysize == 0 {
            return;
        }

        if (*currentScreen).clipBound_X1 >= (*currentScreen).clipBound_X2
            || (*currentScreen).clipBound_Y1 >= (*currentScreen).clipBound_Y2
        {
            return;
        }

        let activePalette = fullPalette[0];
        if (*currentScreen).clipBound_X1 < (*currentScreen).clipBound_X2
            && (*currentScreen).clipBound_Y1 < (*currentScreen).clipBound_Y2
        {
            let lineSize = ((*currentScreen).clipBound_X2 - (*currentScreen).clipBound_X1) >> 4;

            let mut scanline: *const ScanlineInfo =
                scanlines.wrapping_add((*currentScreen).clipBound_Y1 as usize);

            let mut tx = ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) >> 4;
            let mut ty = FROM_FIXED!((*scanline).position.y) >> 4;
            let sheetY = FROM_FIXED!((*scanline).position.y) & 0xF;
            let mut sheetX =
                ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) & 0xF;
            let mut tileRemainX = TILE_SIZE as int32 - sheetX;
            let tileRemainY = TILE_SIZE as int32 - sheetY;

            let mut frameBuffer: *mut uint16 = &mut (*currentScreen).frameBuffer[((*currentScreen)
                .clipBound_X1
                + (*currentScreen).clipBound_Y1 * (*currentScreen).pitch)
                as usize];
            let mut layout = layer
                .layout
                .wrapping_add((tx + (ty << layer.widthShift)) as usize);

            // Remaining pixels on top
            {
                if *layout == 0xFFFF {
                    frameBuffer = frameBuffer.wrapping_add(TILE_SIZE - sheetX as usize);
                } else {
                    let mut pixels: *mut uint8 = &mut tilesetPixels[TILE_DATASIZE
                        * ((*layout & 0xFFF) as usize)
                        + TILE_SIZE * sheetY as usize
                        + sheetX as usize];

                    for _ in 0..tileRemainY {
                        for _ in 0..tileRemainX {
                            if *pixels != 0 {
                                *frameBuffer = activePalette[*pixels as usize];
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(sheetX as usize);
                        frameBuffer = frameBuffer
                            .wrapping_add(((*currentScreen).pitch - tileRemainX) as usize);
                    }

                    frameBuffer = frameBuffer.wrapping_add(
                        (tileRemainX - (*currentScreen).pitch * tileRemainY) as usize,
                    );
                }

                layout = layout.wrapping_add(1);
                tx += 1;
                if tx == layer.xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub(layer.xsize as usize);
                }

                for _ in 0..lineSize {
                    if *layout == 0xFFFF {
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    } else {
                        let mut pixels: *mut uint8 = &mut tilesetPixels[TILE_DATASIZE
                            * (*layout & 0xFFF) as usize
                            + TILE_SIZE * sheetY as usize];
                        for _ in 0..tileRemainY {
                            for i in 0..16 {
                                let index = *pixels.wrapping_add(i);
                                if index != 0 {
                                    *frameBuffer.wrapping_add(i) = activePalette[index as usize];
                                }
                            }
                            frameBuffer = frameBuffer.wrapping_add((*currentScreen).pitch as usize);
                            pixels = pixels.wrapping_add(TILE_SIZE);
                        }

                        frameBuffer = frameBuffer.wrapping_add(
                            TILE_SIZE - ((*currentScreen).pitch * tileRemainY) as usize,
                        );
                    }

                    layout = layout.wrapping_add(1);
                    tx += 1;
                    if tx == layer.xsize as i32 {
                        tx = 0;
                        layout = layout.wrapping_sub(layer.xsize as usize);
                    }
                }

                if *layout == 0xFFFF {
                    frameBuffer =
                        frameBuffer.wrapping_add(((*currentScreen).pitch * tileRemainY) as usize);
                } else {
                    let mut pixels: *mut uint8 = &mut tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + TILE_SIZE * sheetY as usize];

                    for _ in 0..tileRemainY {
                        for _ in 0..sheetX {
                            if *pixels != 0 {
                                *frameBuffer = activePalette[*pixels as usize];
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(tileRemainX as usize);
                        frameBuffer =
                            frameBuffer.wrapping_add(((*currentScreen).pitch - sheetX) as usize);
                    }
                }
            }

            // We've drawn a single line of pixels, increase our variables
            frameBuffer = frameBuffer.wrapping_add(
                sheetX as usize + (-(TILE_SIZE as isize) as usize) * lineSize as usize - TILE_SIZE,
            );
            scanline = scanline.wrapping_add(tileRemainY as usize);
            if ty == layer.ysize as i32 {
                ty = 0;
            } else {
                ty += 1;
            }

            // Draw the bulk of the tiles
            let lineTileCount =
                (((*currentScreen).clipBound_Y2 - (*currentScreen).clipBound_Y1) >> 4) - 1;
            for _ in 0..lineTileCount {
                sheetX =
                    ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) & 0xF;
                tx = ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) >> 4;
                tileRemainX = TILE_SIZE as i32 - sheetX;
                layout = layer
                    .layout
                    .wrapping_add((tx + (ty << layer.widthShift)) as usize);

                // Draw any stray pixels on the left
                if *layout == 0xFFFF {
                    frameBuffer = frameBuffer.wrapping_add(tileRemainX as usize);
                } else {
                    let mut pixels: *mut u8 = &mut tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetX as usize];

                    for _ in 0..TILE_SIZE {
                        for _ in 0..tileRemainX {
                            if *pixels != 0 {
                                *frameBuffer = activePalette[*pixels as usize];
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(sheetX as usize);
                        frameBuffer = frameBuffer
                            .wrapping_add(((*currentScreen).pitch - tileRemainX) as usize);
                    }

                    frameBuffer = frameBuffer.wrapping_add(
                        tileRemainX as usize - TILE_SIZE * (*currentScreen).pitch as usize,
                    );
                }
                layout = layout.wrapping_add(1);
                tx += 1;
                if tx == layer.xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub(layer.xsize as usize);
                }

                // Draw the bulk of the tiles on this line
                for _ in 0..lineSize {
                    if *layout == 0xFFFF {
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    } else {
                        let mut pixels: *mut uint8 =
                            &mut tilesetPixels[TILE_DATASIZE * (*layout & 0xFFF) as usize];

                        for _ in 0..TILE_SIZE {
                            for i in 0..16 {
                                let index = *pixels.wrapping_add(i);
                                if index != 0 {
                                    *frameBuffer.wrapping_add(i) = activePalette[index as usize];
                                }
                            }

                            pixels = pixels.wrapping_add(TILE_SIZE);
                            frameBuffer = frameBuffer.wrapping_add((*currentScreen).pitch as usize);
                        }

                        frameBuffer =
                            frameBuffer.wrapping_sub(TILE_SIZE * (*currentScreen).pitch as usize);
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    }

                    layout = layout.wrapping_add(1);
                    tx += 1;
                    if tx == layer.xsize as int32 {
                        tx = 0;
                        layout = layout.wrapping_sub(layer.xsize as usize);
                    }
                }

                // Draw any stray pixels on the right
                if *layout == 0xFFFF {
                    frameBuffer =
                        frameBuffer.wrapping_add(TILE_SIZE * (*currentScreen).pitch as usize);
                } else {
                    let mut pixels: *mut uint8 =
                        &mut tilesetPixels[TILE_DATASIZE * (*layout & 0xFFF) as usize];

                    for _ in 0..TILE_SIZE {
                        for _ in 0..sheetX {
                            if *pixels != 0 {
                                *frameBuffer = activePalette[*pixels as usize];
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(tileRemainX as usize);
                        frameBuffer =
                            frameBuffer.wrapping_add(((*currentScreen).pitch - sheetX) as usize);
                    }
                }

                // We've drawn a single line, increase our variables
                scanline = scanline.wrapping_add(TILE_SIZE);
                frameBuffer = frameBuffer.wrapping_add(
                    sheetX as usize + (-(TILE_SIZE as isize) as usize) * lineSize as usize
                        - TILE_SIZE,
                );
                if ty == layer.ysize as i32 {
                    ty = 0;
                } else {
                    ty += 1;
                }
            }

            // Remaining pixels on bottom
            {
                tx = ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) >> 4;
                sheetX =
                    ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) & 0xF;
                tileRemainX = TILE_SIZE as i32 - sheetX;
                layout = layer
                    .layout
                    .wrapping_add((tx + (ty << layer.widthShift)) as usize);

                if *layout != 0xFFFF {
                    frameBuffer = frameBuffer.wrapping_add(tileRemainX as usize);
                } else {
                    let mut pixels: *mut uint8 = &mut tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetX as usize];

                    for _ in 0..sheetY {
                        for _ in 0..tileRemainX {
                            if *pixels != 0 {
                                *frameBuffer = activePalette[*pixels as usize];
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(sheetX as usize);
                        frameBuffer = frameBuffer
                            .wrapping_add(((*currentScreen).pitch - tileRemainX) as usize);
                    }

                    frameBuffer = frameBuffer
                        .wrapping_add((tileRemainX - (*currentScreen).pitch * sheetY) as usize);
                }
                layout = layout.wrapping_add(1);
                tx += 1;
                if tx == layer.xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub(layer.xsize as usize);
                }

                for _ in 0..lineSize {
                    if *layout == 0xFFFF {
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    } else {
                        let mut pixels: *mut uint8 =
                            &mut tilesetPixels[TILE_DATASIZE * (*layout & 0xFFF) as usize];
                        for _ in 0..sheetY {
                            for i in 0..16 {
                                let index = *pixels.wrapping_add(i);
                                if index != 0 {
                                    *frameBuffer.wrapping_add(i as usize) =
                                        activePalette[index as usize];
                                }
                            }

                            pixels = pixels.wrapping_add(TILE_SIZE);
                            frameBuffer = frameBuffer.wrapping_add((*currentScreen).pitch as usize);
                        }

                        frameBuffer = frameBuffer
                            .wrapping_add(TILE_SIZE - ((*currentScreen).pitch * sheetY) as usize);
                    }

                    layout = layout.wrapping_sub(1);
                    tx += 1;
                    if tx == layer.xsize as i32 {
                        tx = 0;
                        layout = layout.wrapping_sub(layer.xsize as usize);
                    }
                }

                if *layout != 0xFFFF {
                    let mut pixels: *mut uint8 =
                        &mut tilesetPixels[256 * (*layout & 0xFFF) as usize];

                    for _ in 0..sheetY {
                        for _ in 0..sheetX {
                            if *pixels != 0 {
                                *frameBuffer = activePalette[*pixels as usize];
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(tileRemainX as usize);
                    }
                }
            }
        }
    }
}

#[no_mangle]
#[export_name = "DrawLayerVScroll"]
pub extern "C" fn draw_layer_vscroll(layer: &TileLayer) {
    unsafe {
        if layer.xsize == 0 || !layer.ysize == 0 {
            return;
        }

        let lineTileCount = ((*currentScreen).size.y >> 4) - 1;
        let mut frameBuffer: *mut uint16 =
            &mut (*currentScreen).frameBuffer[(*currentScreen).clipBound_X1 as usize];
        let mut scanline: *const ScanlineInfo =
            scanlines.wrapping_add((*currentScreen).clipBound_X1 as usize);
        let activePalette = &fullPalette[gfxLineBuffer[0] as usize] as *const u16;

        for _ in (*currentScreen).clipBound_X1..(*currentScreen).clipBound_X2 {
            let x = (*scanline).position.x;
            let mut y = (*scanline).position.y;
            let mut ty = FROM_FIXED!(y);

            if ty >= (TILE_SIZE as u16 * layer.ysize) as i32 {
                y -= TO_FIXED!(TILE_SIZE * layer.ysize as usize) as i32;
            } else if ty < 0 {
                y += TO_FIXED!(TILE_SIZE * layer.ysize as usize) as i32;
            }

            let mut tileRemain = TILE_SIZE as i32 - (FROM_FIXED!(y) & 0xF);
            let sheetX = FROM_FIXED!(x) & 0xF;
            let sheetY = FROM_FIXED!(y) & 0xF;
            let mut lineRemain = (*currentScreen).size.y;

            let mut layout: *const uint16 = layer
                .layout
                .wrapping_add(((x >> 20) + ((y >> 20) << layer.widthShift)) as usize);
            lineRemain -= tileRemain;

            if *layout >= 0xFFFF {
                frameBuffer =
                    frameBuffer.wrapping_add(((*currentScreen).pitch * tileRemain) as usize);
            } else {
                let mut pixels = &tilesetPixels[TILE_SIZE
                    * (sheetY as usize + TILE_SIZE * (*layout & 0xFFF) as usize)
                    + sheetX as usize] as *const u8;
                for _ in 0..tileRemain {
                    if *pixels != 0 {
                        *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
                    }
                    pixels = pixels.wrapping_add(TILE_SIZE);
                    frameBuffer = frameBuffer.wrapping_add((*currentScreen).pitch as usize);
                }
            }

            ty = y >> 20;
            for _ in 0..lineTileCount {
                layout = layout.wrapping_add(layer.xsize as usize);

                ty += 1;
                if ty == layer.ysize as i32 {
                    ty = 0;
                    layout = layout.wrapping_sub((layer.ysize << layer.widthShift) as usize);
                }

                if *layout >= 0xFFFF {
                    frameBuffer =
                        frameBuffer.wrapping_add(TILE_SIZE * (*currentScreen).pitch as usize);
                } else {
                    let pixels = &tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetX as usize]
                        as *const u8;

                    for mult in 0..16 {
                        if *pixels.wrapping_add(0x10 * mult) != 0 {
                            *frameBuffer.wrapping_add((*currentScreen).pitch as usize * mult) =
                                *activePalette
                                    .wrapping_add(*pixels.wrapping_add(0x10 * mult) as usize);
                        }
                    }

                    frameBuffer =
                        frameBuffer.wrapping_add((*currentScreen).pitch as usize * TILE_SIZE);
                }

                lineRemain -= TILE_SIZE as i32;
            }

            while lineRemain > 0 {
                layout = layout.wrapping_add(layer.xsize as usize);

                ty += 1;
                if ty == layer.ysize as i32 {
                    ty = 0;
                    layout = layout.wrapping_sub((layer.ysize << layer.widthShift) as usize);
                }

                tileRemain = if lineRemain >= TILE_SIZE as i32 {
                    TILE_SIZE as i32
                } else {
                    lineRemain
                };
                if *layout >= 0xFFFF {
                    frameBuffer =
                        frameBuffer.wrapping_add(((*currentScreen).pitch * sheetY) as usize);
                } else {
                    let mut pixels = &tilesetPixels
                        [TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetX as usize]
                        as *const u8;
                    for _ in 0..tileRemain {
                        if *pixels != 0 {
                            *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
                        }

                        pixels = pixels.wrapping_add(TILE_SIZE);
                        frameBuffer = frameBuffer.wrapping_add((*currentScreen).pitch as usize);
                    }
                }

                lineRemain -= TILE_SIZE as i32;
            }

            frameBuffer = frameBuffer
                .wrapping_sub(((*currentScreen).pitch * (*currentScreen).size.y) as usize);

            scanline = scanline.wrapping_add(1);
            frameBuffer = frameBuffer.wrapping_add(1);
        }
    }
}

#[no_mangle]
#[export_name = "DrawLayerRotozoom"]
pub extern "C" fn draw_layer_rotozoom(layer: &TileLayer) {
    unsafe {
        if layer.xsize == 0 || layer.ysize == 0 {
            return;
        }

        let layout: *const uint16 = layer.layout;
        let frameBuffer = &mut (*currentScreen).frameBuffer[((*currentScreen).clipBound_X1
            + (*currentScreen).clipBound_Y1 * (*currentScreen).pitch)
            as usize..];

        let width = ((TILE_SIZE << layer.widthShift) - 1) as i32;
        let height = ((TILE_SIZE << layer.heightShift) - 1) as i32;
        let lineSize = (*currentScreen).clipBound_X2 - (*currentScreen).clipBound_X1;
        let pitch = (*currentScreen).pitch as usize;
        let clip_top = (*currentScreen).clipBound_Y1;

        for draw_y in clip_top..(*currentScreen).clipBound_Y2 {
            let scanline = scanlines.wrapping_add(draw_y as usize).as_ref().unwrap();
            let activePalette = &fullPalette[gfxLineBuffer[draw_y as usize] as usize];

            for p in 0..lineSize {
                let posX = scanline.position.x + (p * scanline.deform.x);
                let posY = scanline.position.y + (p * scanline.deform.y);
                let tx = posX >> 20;
                let ty = posY >> 20;
                let x = FROM_FIXED!(posX) & 0xF;
                let y = FROM_FIXED!(posY) & 0xF;

                let tile = *layout.wrapping_add(
                    (((width >> 4) & tx) + (((height >> 4) & ty) << layer.widthShift)) as usize,
                ) & 0xFFF;
                let idx = tilesetPixels
                    [TILE_SIZE * (y as usize + TILE_SIZE * tile as usize) + x as usize];

                if idx != 0 {
                    frameBuffer[pitch * (draw_y - clip_top) as usize + p as usize] =
                        activePalette[idx as usize];
                }
            }
        }
    }
}
