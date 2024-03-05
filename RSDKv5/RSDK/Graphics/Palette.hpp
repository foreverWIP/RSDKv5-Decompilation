#ifndef PALETTE_H
#define PALETTE_H

#define PALETTE_BANK_COUNT (0x8)
#define PALETTE_BANK_SIZE  (0x100)

union Color {
    uint8 bytes[4];
    uint32 color;
};

extern "C" {
    extern uint16 rgb32To16_R[0x100];
    extern uint16 rgb32To16_G[0x100];
    extern uint16 rgb32To16_B[0x100];

    extern uint16 globalPalette[PALETTE_BANK_COUNT][PALETTE_BANK_SIZE];
    extern uint16 activeGlobalRows[PALETTE_BANK_COUNT];
    extern uint16 activeStageRows[PALETTE_BANK_COUNT];
    extern uint16 stagePalette[PALETTE_BANK_COUNT][PALETTE_BANK_SIZE];

    extern uint16 fullPalette[PALETTE_BANK_COUNT][PALETTE_BANK_SIZE];

    extern uint8 gfxLineBuffer[SCREEN_YSIZE]; // Pointers to active palette

    extern int32 maskColor;

#if RETRO_REV02
    extern uint16 *tintLookupTable;
#else
    extern uint16 tintLookupTable[0x10000];
#endif
}

namespace RSDK
{

#define RGB888_TO_RGB565(r, g, b) ((b) >> 3) | (((g) >> 2) << 5) | (((r) >> 3) << 11)

#define PACK_RGB888(r, g, b) RGB888_TO_RGB565(r, g, b)


extern "C" {
#if RETRO_REV02
    void LoadPalette(uint8 bankID, const char *filePath, uint16 disabledRows);
#endif
    void SetActivePalette(uint8 newActiveBank, int32 startLine, int32 endLine);
    uint32 GetPaletteEntry(uint8 bankID, uint8 index);
    void SetPaletteEntry(uint8 bankID, uint8 index, uint32 color);
    void SetPaletteMask(uint32 color);
#if RETRO_REV02
    void SetTintLookupTable(uint16 *lookupTable);
#if RETRO_USE_MOD_LOADER && RETRO_MOD_LOADER_VER >= 2
    uint16 *GetTintLookupTable();
#endif
#else
    uint16 *GetTintLookupTable();
#endif
    void CopyPalette(uint8 sourceBank, uint8 srcBankStart, uint8 destinationBank, uint8 destBankStart, uint16 count);
    void RotatePalette(uint8 bankID, uint8 startIndex, uint8 endIndex, bool32 right);
}

extern "C" {
#if RETRO_REV02
    void BlendColors(uint8 destBankID, uint32 *srcColorsA, uint32 *srcColorsB, int32 blendAmount, int32 startIndex, int32 count);
#endif
    void SetPaletteFade(uint8 destBankID, uint8 srcBankA, uint8 srcBankB, int16 blendAmount, int32 startIndex, int32 endIndex);
}

#if RETRO_REV0U
#include "Legacy/PaletteLegacy.hpp"
#endif

} // namespace RSDK

#endif
