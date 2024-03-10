use std::ffi::CStr;

use crate::*;

use self::engine_core::reader::{
    close_file, init_file_info, read_bytes, seek_set, FileModes, LoadFile, DEFAULT_FILEINFO,
};

pub const LEGACY_PALETTE_COUNT: usize = 0x8;
pub const LEGACY_PALETTE_COLOR_COUNT: usize = 0x100;

// Palettes (as RGB565 Colors)
#[no_mangle]
pub static mut Legacy_fullPalette: [[uint16; LEGACY_PALETTE_COLOR_COUNT]; LEGACY_PALETTE_COUNT] =
    [[0; LEGACY_PALETTE_COLOR_COUNT]; LEGACY_PALETTE_COUNT];
#[no_mangle]
pub static mut Legacy_activePalette: &mut [uint16; LEGACY_PALETTE_COLOR_COUNT] =
    unsafe { &mut Legacy_fullPalette[0] }; // Ptr to the 256 color set thats active

#[no_mangle]
pub static mut Legacy_gfxLineBuffer: [uint8; SCREEN_YSIZE * 2] = [0; SCREEN_YSIZE * 2]; // Pointers to active palette
#[no_mangle]
pub static mut Legacy_GFX_LINESIZE: int32 = 0;
#[no_mangle]
pub static mut Legacy_GFX_LINESIZE_MINUSONE: int32 = 0;
#[no_mangle]
pub static mut Legacy_GFX_LINESIZE_DOUBLE: int32 = 0;
#[no_mangle]
pub static mut Legacy_GFX_FRAMEBUFFERSIZE: int32 = 0;
#[no_mangle]
pub static mut Legacy_GFX_FBUFFERMINUSONE: int32 = 0;

#[no_mangle]
pub static mut Legacy_fadeMode: int32 = 0;
#[no_mangle]
pub static mut Legacy_fadeA: uint8 = 0;
#[no_mangle]
pub static mut Legacy_fadeR: uint8 = 0;
#[no_mangle]
pub static mut Legacy_fadeG: uint8 = 0;
#[no_mangle]
pub static mut Legacy_fadeB: uint8 = 0;
#[no_mangle]
pub static mut Legacy_paletteMode: int32 = 1;

#[no_mangle]
#[export_name = "Legacy_SetActivePalette"]
pub extern "C" fn set_active_palette(newActivePal: uint8, startLine: int32, endLine: int32) {
    unsafe {
        if newActivePal < LEGACY_PALETTE_COUNT as u8 {
            for l in startLine..endLine {
                if l as usize >= SCREEN_YSIZE {
                    break;
                }

                Legacy_gfxLineBuffer[l as usize] = newActivePal;
            }
        }

        Legacy_activePalette = &mut Legacy_fullPalette[Legacy_gfxLineBuffer[0] as usize];
    }
}

#[no_mangle]
#[export_name = "Legacy_SetPaletteEntry"]
pub extern "C" fn set_palette_entry(
    paletteIndex: uint8,
    index: uint8,
    r: uint8,
    g: uint8,
    b: uint8,
) {
    unsafe {
        if paletteIndex != 0xFF {
            Legacy_fullPalette[paletteIndex as usize][index as usize] = PACK_RGB888!(r, g, b);
        } else {
            Legacy_activePalette[index as usize] = PACK_RGB888!(r, g, b);
        }
    }
}

#[no_mangle]
#[export_name = "Legacy_SetPaletteEntryPacked"]
pub extern "C" fn set_palette_entry_packed(paletteIndex: uint8, index: uint8, color: uint32) {
    unsafe {
        Legacy_fullPalette[paletteIndex as usize][index as usize] =
            PACK_RGB888!((color >> 16) as u8, (color >> 8) as u8, (color >> 0) as u8);
    }
}

#[no_mangle]
#[export_name = "Legacy_GetPaletteEntryPacked"]
pub extern "C" fn get_palette_entry_packed(bankID: uint8, index: uint8) -> uint32 {
    unsafe {
        // 0xF800 = 1111 1000 0000 0000 = R
        // 0x7E0  = 0000 0111 1110 0000 = G
        // 0x1F   = 0000 0000 0001 1111 = B
        let clr: uint16 = Legacy_fullPalette[bankID as usize & 7][index as usize];

        let R: int32 = (clr as i32 & 0xF800) << 8;
        let G: int32 = (clr as i32 & 0x7E0) << 5;
        let B: int32 = (clr as i32 & 0x1F) << 3;
        return (R | G | B) as u32;
    }
}

#[no_mangle]
#[export_name = "v4_CopyPalette"]
pub extern "C" fn v4_copy_palette(
    sourcePalette: uint8,
    srcPaletteStart: uint8,
    destinationPalette: uint8,
    destPaletteStart: uint8,
    count: uint16,
) {
    if sourcePalette < LEGACY_PALETTE_COUNT as u8 && destinationPalette < LEGACY_PALETTE_COUNT as u8
    {
        unsafe {
            for i in 0..count {
                Legacy_fullPalette[destinationPalette as usize]
                    [(destPaletteStart as u16 + i) as usize] = Legacy_fullPalette
                    [sourcePalette as usize][(srcPaletteStart as u16 + i) as usize];
            }
        }
    }
}

#[no_mangle]
#[export_name = "v3_CopyPalette"]
pub extern "C" fn v3_copy_palette(sourcePalette: uint8, destinationPalette: uint8) {
    v4_copy_palette(
        sourcePalette,
        0,
        destinationPalette,
        0,
        LEGACY_PALETTE_COLOR_COUNT as u16,
    );
}

#[no_mangle]
#[export_name = "v4_RotatePalette"]
pub extern "C" fn v4_rotate_palette(palID: int32, startIndex: uint8, endIndex: uint8, right: bool) {
    unsafe {
        let palette: &mut [u16; LEGACY_PALETTE_COLOR_COUNT];
        if palID == -1 {
            palette = Legacy_activePalette;
        } else {
            palette = &mut Legacy_fullPalette[palID as usize];
        }
        if right {
            let startClr: uint16 = palette[endIndex as usize];
            let mut i: i32 = endIndex as i32;
            loop {
                if i <= startIndex as i32 {
                    break;
                }

                palette[i as usize] = palette[i as usize - 1];

                i -= 1;
            }
            palette[startIndex as usize] = startClr;
        } else {
            let startClr: uint16 = palette[startIndex as usize];
            for i in startIndex..endIndex {
                palette[i as usize] = palette[i as usize + 1];
            }
            palette[endIndex as usize] = startClr;
        }
    }
}

#[no_mangle]
#[export_name = "v3_RotatePalette"]
pub extern "C" fn v3_rotate_palette(startIndex: uint8, endIndex: uint8, right: bool) {
    v4_rotate_palette(-1, startIndex, endIndex, right);
}

#[no_mangle]
#[export_name = "Legacy_SetFade"]
pub extern "C" fn set_fade(R: uint8, G: uint8, B: uint8, A: uint16) {
    unsafe {
        Legacy_fadeMode = 1;
        Legacy_fadeR = R;
        Legacy_fadeG = G;
        Legacy_fadeB = B;
        Legacy_fadeA = if A > 0xFF { 0xFF } else { A as u8 };
    }
}

#[no_mangle]
#[export_name = "Legacy_LoadPalette"]
pub extern "C" fn load_palette(
    filePath: *const i8,
    mut paletteID: int32,
    mut startPaletteIndex: int32,
    startIndex: int32,
    endIndex: int32,
) {
    let fullPath =
        "Data/Palettes/".to_owned() + unsafe { CStr::from_ptr(filePath).to_str().unwrap() } + "\0";

    let mut info = DEFAULT_FILEINFO;
    init_file_info(&mut info);

    unsafe {
        if LoadFile(
            &mut info,
            fullPath.as_ptr() as *const i8,
            FileModes::FMODE_RB as u8,
        ) == true32
        {
            seek_set(&mut info, 3 * startIndex);

            if paletteID >= LEGACY_PALETTE_COUNT as i32 || paletteID < 0 {
                paletteID = 0;
            }

            let mut color = [0u8; 3];
            if paletteID != 0 {
                for _ in startIndex..endIndex {
                    read_bytes(&mut info, (&mut color) as *mut u8, 3);
                    set_palette_entry(
                        paletteID as u8,
                        startPaletteIndex as u8,
                        color[0],
                        color[1],
                        color[2],
                    );
                    startPaletteIndex += 1;
                }
            } else {
                for _ in startIndex..endIndex {
                    read_bytes(&mut info, (&mut color) as *mut u8, 3);
                    set_palette_entry(0xff, startPaletteIndex as u8, color[0], color[1], color[2]);
                    startPaletteIndex += 1;
                }
            }

            close_file(&mut info);
        }
    }
}

#[no_mangle]
#[export_name = "Legacy_SetPaletteFade"]
pub extern "C" fn set_palette_fade(
    destPaletteID: uint8,
    srcPaletteA: uint8,
    srcPaletteB: uint8,
    mut blendAmount: uint16,
    startIndex: int32,
    endIndex: int32,
) {
    if destPaletteID >= LEGACY_PALETTE_COUNT as u8
        || srcPaletteA >= LEGACY_PALETTE_COUNT as u8
        || srcPaletteB >= LEGACY_PALETTE_COUNT as u8
    {
        return;
    }

    if blendAmount >= 0x100 {
        blendAmount = 0xFF;
    }

    if startIndex >= endIndex {
        return;
    }

    let blendA: uint32 = 0xFF - blendAmount as u32;
    unsafe {
        let mut paletteColor: *mut uint16 =
            &mut Legacy_fullPalette[destPaletteID as usize][startIndex as usize];
        for i in startIndex..endIndex {
            let clrA: uint32 = get_palette_entry_packed(srcPaletteA, i as u8);
            let clrB: uint32 = get_palette_entry_packed(srcPaletteB, i as u8);

            let r: int32 = (blendAmount as u32 * ((clrB >> 0x10) & 0xFF)
                + blendA * ((clrA >> 0x10) & 0xFF)) as i32;
            let g: int32 = (blendAmount as u32 * ((clrB >> 0x08) & 0xFF)
                + blendA * ((clrA >> 0x08) & 0xFF)) as i32;
            let b: int32 = (blendAmount as u32 * ((clrB >> 0x00) & 0xFF)
                + blendA * ((clrA >> 0x00) & 0xFF)) as i32;

            *paletteColor = PACK_RGB888!((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8);

            paletteColor = paletteColor.wrapping_add(1);
        }
    }
}

#[no_mangle]
#[export_name = "Legacy_SetLimitedFade"]
pub extern "C" fn set_limited_fade(
    paletteID: uint8,
    R: uint8,
    G: uint8,
    B: uint8,
    mut blendAmount: uint16,
    startIndex: int32,
    endIndex: int32,
) {
    if (paletteID >= LEGACY_PALETTE_COUNT as u8) {
        return;
    }

    unsafe {
        Legacy_paletteMode = 1;
        Legacy_activePalette = &mut Legacy_fullPalette[paletteID as usize];
    }

    if (blendAmount >= 0x100) {
        blendAmount = 0xFF;
    }

    if (startIndex >= endIndex) {
        return;
    }

    let blendA: uint32 = 0xFF - blendAmount as u32;
    unsafe {
        let mut paletteColor: *mut uint16 =
            &mut Legacy_fullPalette[paletteID as usize][startIndex as usize];
        for i in startIndex..endIndex {
            let clrA: uint32 = get_palette_entry_packed(paletteID, i as u8);

            let r: int32 =
                (blendAmount as u32 * R as u32 + blendA * ((clrA >> 0x10) & 0xFF)) as i32;
            let g: int32 =
                (blendAmount as u32 * G as u32 + blendA * ((clrA >> 0x08) & 0xFF)) as i32;
            let b: int32 =
                (blendAmount as u32 * B as u32 + blendA * ((clrA >> 0x00) & 0xFF)) as i32;

            *paletteColor = PACK_RGB888!((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8);

            paletteColor = paletteColor.wrapping_add(1);
        }
    }
}
