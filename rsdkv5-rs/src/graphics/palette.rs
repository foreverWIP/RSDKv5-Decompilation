use crate::*;

const PALETTE_BANK_COUNT: usize = 0x8;
const PALETTE_BANK_SIZE: usize = 0x100;

// #define RGB888_TO_RGB565(r, g, b) ((b) >> 3) | (((g) >> 2) << 5) | (((r) >> 3) << 11)
#[macro_export]
macro_rules! RGB888_TO_RGB565 {
    ($r:expr, $g:expr, $b:expr) => {
        (($b) as u16 >> 3) | ((($g) as u16 >> 2) << 5) | ((($r) as u16 >> 3) << 11)
    };
}
// #define PACK_RGB888(r, g, b) RGB888_TO_RGB565(r, g, b)
#[macro_export]
macro_rules! PACK_RGB888 {
    ($r:expr, $g:expr, $b:expr) => {
        RGB888_TO_RGB565!($r, $g, $b)
    };
}

#[no_mangle]
static mut rgb32To16_R: [uint16; 0x100] = [0; 0x100];
#[no_mangle]
static mut rgb32To16_G: [uint16; 0x100] = [0; 0x100];
#[no_mangle]
static mut rgb32To16_B: [uint16; 0x100] = [0; 0x100];

#[no_mangle]
static mut globalPalette: [[uint16; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT] =
    [[0; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT];
#[no_mangle]
static mut activeGlobalRows: [uint16; PALETTE_BANK_COUNT] = [0; PALETTE_BANK_COUNT];
#[no_mangle]
static mut activeStageRows: [uint16; PALETTE_BANK_COUNT] = [0; PALETTE_BANK_COUNT];
#[no_mangle]
static mut stagePalette: [[uint16; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT] =
    [[0; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT];

#[no_mangle]
pub static mut fullPalette: [[uint16; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT] =
    [[0; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT];
#[no_mangle]
pub static mut gfxLineBuffer: [uint8; SCREEN_YSIZE * 2] = [0; SCREEN_YSIZE * 2];
#[no_mangle]
static mut maskColor: int32 = 0;

cfg_if::cfg_if! {
    if #[cfg(feature = "version_2")] {
        #[no_mangle]
        static mut tintLookupTable: *const uint16 = std::ptr::null_mut();
        #[cfg(feature = "version_u")]
        #[no_mangle]
        static mut defaultTintLookupTable: [uint16; 0x10000] = [0; 0x10000];
    } else {
        #[no_mangle]
        static mut tintLookupTable: [uint16; 0x10000] = [0; 0x10000];
    }
}

#[no_mangle]
#[export_name = "SetActivePalette"]
pub extern "C" fn set_active_palette(newActiveBank: uint8, startLine: int32, endLine: int32) {
    if newActiveBank < PALETTE_BANK_COUNT as u8 {
        unsafe {
            for l in startLine..endLine {
                if l >= SCREEN_YSIZE as i32 {
                    break;
                }
                gfxLineBuffer[l as usize] = newActiveBank;
            }
        }
    }
}

#[no_mangle]
#[export_name = "GetPaletteEntry"]
pub extern "C" fn get_palette_entry(bankID: uint8, index: uint8) -> uint32 {
    // 0xF800 = 1111 1000 0000 0000 = R
    // 0x7E0  = 0000 0111 1110 0000 = G
    // 0x1F   = 0000 0000 0001 1111 = B
    let clr: uint16 = unsafe { fullPalette[(bankID & 7) as usize][index as usize] };

    let R: int32 = (clr as i32 & 0xF800) << 8;
    let G: int32 = (clr as i32 & 0x7E0) << 5;
    let B: int32 = (clr as i32 & 0x1F) << 3;
    return (R | G | B) as u32;
}

#[no_mangle]
#[export_name = "SetPaletteEntry"]
pub extern "C" fn set_palette_entry(bankID: uint8, index: uint8, color: uint32) {
    let color = color as usize;
    unsafe {
        fullPalette[bankID as usize][index as usize] = rgb32To16_B[(color >> 0) & 0xFF]
            | rgb32To16_G[(color >> 8) & 0xFF]
            | rgb32To16_R[(color >> 16) & 0xFF];
    }
}

#[no_mangle]
#[export_name = "SetPaletteMask"]
pub extern "C" fn set_palette_mask(color: uint32) {
    let color = color as usize;
    unsafe {
        maskColor = (rgb32To16_B[(color >> 0) & 0xFF]
            | rgb32To16_G[(color >> 8) & 0xFF]
            | rgb32To16_R[(color >> 16) & 0xFF]) as i32;
    }
}

#[cfg(feature = "version_2")]
#[no_mangle]
#[export_name = "SetTintLookupTable"]
pub extern "C" fn set_tint_lookup_table(lookupTable: *const uint16) {
    unsafe {
        tintLookupTable = lookupTable as *const u16;
    }
}

#[cfg(any(
    not(feature = "version_2"),
    all(feature = "version_2", feature = "mod_loader_v2")
))]
#[no_mangle]
#[export_name = "GetTintLookupTable"]
pub extern "C" fn get_tint_lookup_table() -> *const uint16 {
    unsafe {
        return tintLookupTable;
    }
}

#[no_mangle]
#[export_name = "CopyPalette"]
pub extern "C" fn copy_palette(
    sourceBank: uint8,
    srcBankStart: uint8,
    destinationBank: uint8,
    destBankStart: uint8,
    count: uint16,
) {
    if sourceBank < PALETTE_BANK_COUNT as u8 && destinationBank < PALETTE_BANK_COUNT as u8 {
        unsafe {
            for i in 0..count {
                fullPalette[destinationBank as usize][(destBankStart + i as u8) as usize] =
                    fullPalette[sourceBank as usize][(srcBankStart + i as u8) as usize];
            }
        }
    }
}

#[no_mangle]
#[export_name = "RotatePalette"]
pub extern "C" fn rotate_palette(bankID: uint8, startIndex: uint8, endIndex: uint8, right: bool32) {
    let bankID = bankID as usize;
    let startIndex = startIndex as usize;
    let endIndex = endIndex as usize;
    unsafe {
        if (right == true32) {
            let startClr: uint16 = fullPalette[bankID][endIndex];
            // for (int32 i = endIndex; i > startIndex; --i)
            let mut i = endIndex;
            loop {
                fullPalette[bankID][i] = fullPalette[bankID][i - 1];
                i -= 1;
                if i <= startIndex {
                    break;
                }
            }
            fullPalette[bankID][startIndex] = startClr;
        } else {
            let startClr: uint16 = fullPalette[bankID][startIndex];
            for i in startIndex..endIndex {
                fullPalette[bankID][i] = fullPalette[bankID][i + 1];
            }
            fullPalette[bankID][endIndex] = startClr;
        }
    }
}

#[cfg(feature = "version_2")]
#[no_mangle]
#[export_name = "LoadPalette"]
pub extern "C" fn load_palette(bankID: uint8, filename: *const i8, disabledRows: uint16) {
    use crate::engine_core::reader::{
        close_file, load_file, read_int_8, seek_cur, FileInfo, FileModes, DEFAULT_FILEINFO,
    };

    use self::engine_core::reader::init_file_info;

    unsafe {
        let fullFilePath = "Data/Palettes/".to_owned() + &to_string(filename) + "\0";

        let mut info = DEFAULT_FILEINFO;
        init_file_info(&mut info);
        if (load_file(
            &mut info,
            fullFilePath.as_ptr() as *const i8,
            FileModes::FMODE_RB as u8,
        ) == true32)
        {
            for r in 0..0x10 {
                if (disabledRows >> r & 1) == 0 {
                    for c in 0..0x10 {
                        let red: uint8 = read_int_8(&mut info);
                        let green: uint8 = read_int_8(&mut info);
                        let blue: uint8 = read_int_8(&mut info);
                        fullPalette[bankID as usize][(r << 4) + c] = rgb32To16_B[blue as usize]
                            | rgb32To16_G[green as usize]
                            | rgb32To16_R[red as usize];
                    }
                } else {
                    seek_cur(&mut info, 0x10 * 3);
                }
            }

            close_file(&mut info);
        }
    }
}

#[cfg(feature = "version_2")]
#[no_mangle]
#[export_name = "BlendColors"]
pub extern "C" fn blend_colors(
    destBankID: uint8,
    mut srcColorsA: *const uint32,
    mut srcColorsB: *const uint32,
    mut blendAmount: int32,
    startIndex: int32,
    count: int32,
) {
    if (destBankID >= PALETTE_BANK_COUNT as u8 || srcColorsA.is_null() || srcColorsB.is_null()) {
        return;
    }

    blendAmount = blendAmount.clamp(0x00, 0xFF);

    let blendA: uint8 = 0xFF - blendAmount as u8;
    unsafe {
        let mut paletteColor: *mut uint16 =
            &mut fullPalette[destBankID as usize][startIndex as usize];
        for i in startIndex..(startIndex + count) {
            let r: int32 = blendAmount * ((*srcColorsB >> 0x10) & 0xFF) as i32
                + blendA as i32 * ((*srcColorsA >> 0x10) & 0xFF) as i32;
            let g: int32 = blendAmount * ((*srcColorsB >> 0x08) & 0xFF) as i32
                + blendA as i32 * ((*srcColorsA >> 0x08) & 0xFF) as i32;
            let b: int32 = blendAmount * ((*srcColorsB >> 0x00) & 0xFF) as i32
                + blendA as i32 * ((*srcColorsA >> 0x00) & 0xFF) as i32;

            *paletteColor = PACK_RGB888!((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8);

            srcColorsA = srcColorsA.wrapping_add(1);
            srcColorsB = srcColorsB.wrapping_add(1);
            paletteColor = paletteColor.wrapping_add(1);
        }
    }
}

#[no_mangle]
#[export_name = "SetPaletteFade"]
pub extern "C" fn set_palette_fade(
    destBankID: uint8,
    srcBankA: uint8,
    srcBankB: uint8,
    mut blendAmount: int16,
    startIndex: int32,
    mut endIndex: int32,
) {
    if (destBankID >= PALETTE_BANK_COUNT as u8
        || srcBankA >= PALETTE_BANK_COUNT as u8
        || srcBankB >= PALETTE_BANK_COUNT as u8)
    {
        return;
    }

    blendAmount = blendAmount.clamp(0x00, 0xFF);
    endIndex = endIndex.min(0x100);

    if (startIndex >= endIndex) {
        return;
    }

    let blendA: uint32 = (0xFF - blendAmount) as u32;
    unsafe {
        let mut paletteColor: *mut uint16 =
            &mut fullPalette[destBankID as usize][startIndex as usize];
        for i in startIndex..(endIndex + 1) {
            let clrA: uint32 = get_palette_entry(srcBankA, i as u8);
            let clrB: uint32 = get_palette_entry(srcBankB, i as u8);

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
