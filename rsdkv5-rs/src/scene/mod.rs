use crate::*;

use self::{
    graphics::{
        drawing::{currentScreen, ScreenInfo, CAMERA_COUNT},
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
pub struct ScrollInfo {
    tilePos: int32,
    parallaxFactor: int32,
    scrollSpeed: int32,
    scrollPos: int32,
    deform: uint8,
    unknown: uint8, // stored in the scene, but always 0, never referenced in-engine either...
}

#[repr(C)]
pub struct ScanlineInfo {
    position: Vector2, // position of the scanline
    deform: Vector2,   // deformation that should be applied (only applies to RotoZoom type)
}

#[repr(C)]
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
    // void (*scanlineCallback)(ScanlineInfo *scanlines);
    scanlineCallback: extern "C" fn(*mut ScanlineInfo),
    scrollInfoCount: uint16,
    scrollInfo: [ScrollInfo; 0x100],
    name: HashMD5,
    layout: *mut uint16,
    lineScroll: *mut uint8,
}

#[no_mangle]
pub static mut scanlines: *mut ScanlineInfo = std::ptr::null_mut();
#[no_mangle]
pub static mut tilesetPixels: [uint8; TILESET_SIZE * 4] = [0; TILESET_SIZE * 4];

#[no_mangle]
#[export_name = "DrawLayerHScroll"]
pub extern "C" fn draw_layer_hscroll(layer: *const TileLayer) {
    unsafe {
        if ((*layer).xsize == 0 || (*layer).ysize == 0) {
            return;
        }

        let lineTileCount: int32 = ((*currentScreen).pitch >> 4) - 1;
        let mut lineBuffer: *const uint8 = gfxLineBuffer
            .as_ptr()
            .wrapping_add((*currentScreen).clipBound_Y1 as usize);
        let mut scanline: *mut ScanlineInfo =
            scanlines.wrapping_add((*currentScreen).clipBound_Y1 as usize);
        let mut frameBuffer: *mut uint16 = (*currentScreen)
            .frameBuffer
            .as_mut_ptr()
            .wrapping_add(((*currentScreen).pitch * (*currentScreen).clipBound_Y1) as usize);

        for cy in (*currentScreen).clipBound_Y1..((*currentScreen).clipBound_Y2 + 1) {
            let mut x: int32 = (*scanline).position.x;
            let y: int32 = (*scanline).position.y;
            let tileX: int32 = FROM_FIXED!(x);
            let activePalette: *const uint16 =
                (fullPalette.as_ptr() as *const uint16).wrapping_add(*lineBuffer as usize);
            lineBuffer = lineBuffer.wrapping_add(1);

            if ((tileX as usize) >= TILE_SIZE * ((*layer).xsize as usize)) {
                x = TO_FIXED!(tileX - (TILE_SIZE * ((*layer).xsize as usize)) as i32);
            } else if (tileX < 0) {
                x = TO_FIXED!(tileX + (TILE_SIZE * ((*layer).xsize as usize)) as i32);
            }

            let mut tileRemain: int32 = (TILE_SIZE - (FROM_FIXED!(x) & 0xF) as usize) as i32;
            let sheetX: int32 = FROM_FIXED!(x) & 0xF;
            let sheetY: int32 = (TILE_SIZE * (FROM_FIXED!(y) & 0xF) as usize) as i32;
            let mut lineRemain: int32 = (*currentScreen).pitch;

            let mut tx: int32 = x >> 20;
            let mut layout: *mut uint16 = (*layer)
                .layout
                .wrapping_add((tx + ((y >> 20) << (*layer).widthShift)) as usize);
            lineRemain -= tileRemain;

            if (*layout >= 0xFFFF) {
                frameBuffer = frameBuffer.wrapping_add(tileRemain as usize);
            } else {
                let mut pixels: *mut uint8 = tilesetPixels.as_mut_ptr().wrapping_add(
                    TILE_DATASIZE * ((*layout & 0xFFF) as usize) + (sheetY + sheetX) as usize,
                );
                for x in 0..tileRemain {
                    if (*pixels != 0) {
                        *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
                    }
                    pixels = pixels.wrapping_add(1);
                    frameBuffer = frameBuffer.wrapping_add(1);
                }
            }

            for l in 0..lineTileCount {
                layout = layout.wrapping_add(1);

                if (tx == (*layer).xsize as i32) {
                    tx = 0;
                    layout = layout.wrapping_sub((*layer).xsize as usize);
                } else {
                    tx += 1;
                }

                if (*layout < 0xFFFF) {
                    let mut pixels: *mut uint8 = tilesetPixels
                        .as_mut_ptr()
                        .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetY as usize);

                    for p in 0..16 {
                        let index = *pixels.wrapping_add(p);
                        if index != 0 {
                            *frameBuffer.wrapping_add(p) =
                                *activePalette.wrapping_add(index as usize);
                        }
                    }
                }

                frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                lineRemain = lineRemain.wrapping_sub(TILE_SIZE as i32);
            }

            while (lineRemain > 0) {
                layout = layout.wrapping_add(1);

                if (tx == (*layer).xsize as i32) {
                    tx = 0;
                    layout = layout.wrapping_sub((*layer).xsize as usize);
                } else {
                    tx += 1;
                }

                tileRemain = if lineRemain >= TILE_SIZE as i32 {
                    TILE_SIZE as i32
                } else {
                    lineRemain
                };

                if (*layout >= 0xFFFF) {
                    frameBuffer = frameBuffer.wrapping_add(tileRemain as usize);
                } else {
                    let mut pixels: *mut uint8 = tilesetPixels
                        .as_mut_ptr()
                        .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetY as usize);
                    for x in 0..tileRemain {
                        if (*pixels != 0) {
                            *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
                        }
                        pixels = pixels.wrapping_add(1);
                        frameBuffer = frameBuffer.wrapping_add(1);
                    }
                }

                lineRemain = lineRemain.wrapping_sub(TILE_SIZE as i32);
            }

            scanline = scanline.wrapping_add(1);
        }
    }
}

#[no_mangle]
#[export_name = "DrawLayerBasic"]
pub extern "C" fn draw_layer_basic(layer: *const TileLayer) {
    unsafe {
        if ((*layer).xsize == 0 || (*layer).ysize == 0) {
            return;
        }

        if ((*currentScreen).clipBound_X1 >= (*currentScreen).clipBound_X2
            || (*currentScreen).clipBound_Y1 >= (*currentScreen).clipBound_Y2)
        {
            return;
        }

        let mut activePalette: *mut uint16 = fullPalette.as_mut_ptr() as *mut u16;
        if ((*currentScreen).clipBound_X1 < (*currentScreen).clipBound_X2
            && (*currentScreen).clipBound_Y1 < (*currentScreen).clipBound_Y2)
        {
            let mut lineSize: int32 =
                ((*currentScreen).clipBound_X2 - (*currentScreen).clipBound_X1) >> 4;

            let mut scanline: *mut ScanlineInfo =
                scanlines.wrapping_add((*currentScreen).clipBound_Y1 as usize);

            let mut tx: int32 =
                ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) >> 4;
            let mut ty: int32 = FROM_FIXED!((*scanline).position.y) >> 4;
            let mut sheetY: int32 = FROM_FIXED!((*scanline).position.y) & 0xF;
            let mut sheetX: int32 =
                ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) & 0xF;
            let mut tileRemainX: int32 = TILE_SIZE as int32 - sheetX;
            let mut tileRemainY: int32 = TILE_SIZE as int32 - sheetY;

            let mut frameBuffer: *mut uint16 =
                (*currentScreen).frameBuffer.as_mut_ptr().wrapping_add(
                    ((*currentScreen).clipBound_X1
                        + (*currentScreen).clipBound_Y1 * (*currentScreen).pitch)
                        as usize,
                );
            let mut layout: *mut uint16 = (*layer)
                .layout
                .wrapping_add((tx + (ty << (*layer).widthShift)) as usize);

            // Remaining pixels on top
            {
                if (*layout == 0xFFFF) {
                    frameBuffer = frameBuffer.wrapping_add(TILE_SIZE - sheetX as usize);
                } else {
                    let mut pixels: *mut uint8 = tilesetPixels.as_mut_ptr().wrapping_add(
                        TILE_DATASIZE * ((*layout & 0xFFF) as usize)
                            + TILE_SIZE * sheetY as usize
                            + sheetX as usize,
                    );

                    for y in 0..tileRemainY {
                        for x in 0..tileRemainX {
                            if (*pixels != 0) {
                                *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
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
                if (tx == (*layer).xsize as i32) {
                    tx = 0;
                    layout = layout.wrapping_sub((*layer).xsize as usize);
                } else {
                    tx += 1;
                }

                for x in 0..lineSize {
                    if (*layout == 0xFFFF) {
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    } else {
                        let mut pixels: *mut uint8 = tilesetPixels.as_mut_ptr().wrapping_add(
                            TILE_DATASIZE * (*layout & 0xFFF) as usize
                                + TILE_SIZE * sheetY as usize,
                        );
                        for y in 0..tileRemainY {
                            for i in 0..16 {
                                let index = *pixels.wrapping_add(i);
                                if index != 0 {
                                    *frameBuffer.wrapping_add(i) =
                                        *activePalette.wrapping_add(index as usize);
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
                    if (tx == (*layer).xsize as i32) {
                        tx = 0;
                        layout = layout.wrapping_sub((*layer).xsize as usize);
                    } else {
                        tx += 1;
                    }
                }

                if (*layout == 0xFFFF) {
                    frameBuffer =
                        frameBuffer.wrapping_add(((*currentScreen).pitch * tileRemainY) as usize);
                } else {
                    let mut pixels: *mut uint8 = tilesetPixels.as_mut_ptr().wrapping_add(
                        TILE_DATASIZE * (*layout & 0xFFF) as usize + TILE_SIZE * sheetY as usize,
                    );

                    for y in 0..tileRemainY {
                        for x in 0..sheetX {
                            if (*pixels != 0) {
                                *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
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
            if (ty == (*layer).ysize as i32) {
                ty = 0;
            } else {
                ty += 1;
            }

            // Draw the bulk of the tiles
            let mut lineTileCount: int32 =
                (((*currentScreen).clipBound_Y2 - (*currentScreen).clipBound_Y1) >> 4) - 1;
            for l in 0..lineTileCount {
                sheetX =
                    ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) & 0xF;
                tx = ((*currentScreen).clipBound_X1 + FROM_FIXED!((*scanline).position.x)) >> 4;
                tileRemainX = TILE_SIZE as i32 - sheetX;
                layout = (*layer)
                    .layout
                    .wrapping_add((tx + (ty << (*layer).widthShift)) as usize);

                // Draw any stray pixels on the left
                if (*layout == 0xFFFF) {
                    frameBuffer = frameBuffer.wrapping_add(tileRemainX as usize);
                } else {
                    let mut pixels: *mut u8 = tilesetPixels
                        .as_mut_ptr()
                        .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetX as usize);

                    for y in 0..TILE_SIZE {
                        for x in 0..tileRemainX {
                            if (*pixels != 0) {
                                *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
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
                if tx == (*layer).xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub((*layer).xsize as usize);
                } else {
                    tx += 1;
                }

                // Draw the bulk of the tiles on this line
                for x in 0..lineSize {
                    if (*layout == 0xFFFF) {
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    } else {
                        let mut pixels: *mut uint8 = tilesetPixels
                            .as_mut_ptr()
                            .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize);

                        for y in 0..TILE_SIZE {
                            for i in 0..16 {
                                let index = *pixels.wrapping_add(i);
                                if index != 0 {
                                    *frameBuffer.wrapping_add(i) =
                                        *activePalette.wrapping_add(index as usize);
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
                    if (tx == (*layer).xsize as int32) {
                        tx = 0;
                        layout = layout.wrapping_sub((*layer).xsize as usize);
                    } else {
                        tx += 1;
                    }
                }

                // Draw any stray pixels on the right
                if (*layout == 0xFFFF) {
                    frameBuffer =
                        frameBuffer.wrapping_add(TILE_SIZE * (*currentScreen).pitch as usize);
                } else {
                    let mut pixels: *mut uint8 = tilesetPixels
                        .as_mut_ptr()
                        .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize);

                    for y in 0..TILE_SIZE {
                        for x in 0..sheetX {
                            if (*pixels != 0) {
                                *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(tileRemainX as usize);
                        frameBuffer =
                            frameBuffer.wrapping_add(((*currentScreen).pitch - sheetX) as usize);
                    }
                }
                layout = layout.wrapping_add(1);
                if (tx == (*layer).xsize as i32) {
                    tx = 0;
                    layout = layout.wrapping_sub((*layer).xsize as usize);
                } else {
                    tx += 1
                }

                // We've drawn a single line, increase our variables
                scanline = scanline.wrapping_add(TILE_SIZE);
                frameBuffer = frameBuffer.wrapping_add(
                    sheetX as usize + (-(TILE_SIZE as isize) as usize) * lineSize as usize
                        - TILE_SIZE,
                );
                if (ty == (*layer).ysize as i32) {
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
                layout = (*layer)
                    .layout
                    .wrapping_add((tx + (ty << (*layer).widthShift)) as usize);

                if (*layout != 0xFFFF) {
                    frameBuffer = frameBuffer.wrapping_add(tileRemainX as usize);
                } else {
                    let mut pixels: *mut uint8 = tilesetPixels
                        .as_mut_ptr()
                        .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize + sheetX as usize);

                    for y in 0..sheetY {
                        for x in 0..tileRemainX {
                            if (*pixels != 0) {
                                *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
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
                if tx == (*layer).xsize as i32 {
                    tx = 0;
                    layout = layout.wrapping_sub((*layer).xsize as usize);
                } else {
                    tx += 1;
                }

                for x in 0..lineSize {
                    if (*layout == 0xFFFF) {
                        frameBuffer = frameBuffer.wrapping_add(TILE_SIZE);
                    } else {
                        let mut pixels: *mut uint8 = tilesetPixels
                            .as_mut_ptr()
                            .wrapping_add(TILE_DATASIZE * (*layout & 0xFFF) as usize);
                        for y in 0..sheetY {
                            for i in 0..16 {
                                let index = *pixels.wrapping_add(i);
                                if index != 0 {
                                    *frameBuffer.wrapping_add(i as usize) =
                                        *activePalette.wrapping_add(index as usize);
                                }
                            }

                            pixels = pixels.wrapping_add(TILE_SIZE);
                            frameBuffer = frameBuffer.wrapping_add((*currentScreen).pitch as usize);
                        }

                        frameBuffer = frameBuffer
                            .wrapping_add(TILE_SIZE - ((*currentScreen).pitch * sheetY) as usize);
                    }

                    layout = layout.wrapping_sub(1);
                    if (tx == (*layer).xsize as i32) {
                        tx = 0;
                        layout = layout.wrapping_sub((*layer).xsize as usize);
                    } else {
                        tx += 1;
                    }
                }

                if (*layout != 0xFFFF) {
                    let mut pixels: *mut uint8 = tilesetPixels
                        .as_mut_ptr()
                        .wrapping_add(256 * (*layout & 0xFFF) as usize);

                    for y in 0..sheetY {
                        for x in 0..sheetX {
                            if (*pixels != 0) {
                                *frameBuffer = *activePalette.wrapping_add(*pixels as usize);
                            }
                            pixels = pixels.wrapping_add(1);
                            frameBuffer = frameBuffer.wrapping_add(1);
                        }

                        pixels = pixels.wrapping_add(tileRemainX as usize);
                    }

                    frameBuffer =
                        frameBuffer.wrapping_add(((*currentScreen).pitch - sheetX) as usize);
                }
            }
        }
    }
}
