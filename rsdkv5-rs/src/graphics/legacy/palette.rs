use crate::*;

pub const LEGACY_PALETTE_COUNT: usize = 0x8;
pub const LEGACY_PALETTE_COLOR_COUNT: usize = 0x100;

// Palettes (as RGB565 Colors)
#[no_mangle]
pub static mut Legacy_fullPalette: [[uint16; LEGACY_PALETTE_COLOR_COUNT]; LEGACY_PALETTE_COUNT] =
    [[0; LEGACY_PALETTE_COLOR_COUNT]; LEGACY_PALETTE_COUNT];
#[no_mangle]
pub static mut Legacy_activePalette: *const uint16 =
    unsafe { &Legacy_fullPalette[0] as *const u16 }; // Ptr to the 256 color set thats active

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
