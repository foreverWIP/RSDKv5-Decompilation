use crate::*;

const PALETTE_BANK_COUNT: usize = 0x8;
const PALETTE_BANK_SIZE: usize = 0x100;

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
static mut fullPalette: [[uint16; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT] =
    [[0; PALETTE_BANK_SIZE]; PALETTE_BANK_COUNT];
#[no_mangle]
static mut gfxLineBuffer: [uint8; SCREEN_YSIZE] = [0; SCREEN_YSIZE];
#[no_mangle]
static mut maskColor: int32 = 0;

/*#if RETRO_REV02
uint16 *RSDK::tintLookupTable = NULL;
#else
uint16 RSDK::tintLookupTable[0x10000];
#endif*/

cfg_if::cfg_if! {
    if #[cfg(feature = "version_2")] {
        #[no_mangle]
        static mut tintLookupTable: *mut uint16 = std::ptr::null_mut();
    } else {
        #[no_mangle]
        static mut tintLookupTable: [uint16; 0x10000] = [0; 0x10000];
    }
}
