#define LEGACY_PALETTE_COUNT       (0x8)
#define LEGACY_PALETTE_COLOR_COUNT (0x100)

extern "C" {
    // Palettes (as RGB565 Colors)
    extern uint16 *Legacy_activePalette; // Pointers to the 256 color set thats active

    extern uint8 Legacy_gfxLineBuffer[SCREEN_YSIZE * 2]; // Pointers to active palette
    extern int32 Legacy_GFX_LINESIZE;
    extern int32 Legacy_GFX_LINESIZE_MINUSONE;
    extern int32 Legacy_GFX_LINESIZE_DOUBLE;
    extern int32 Legacy_GFX_FRAMEBUFFERSIZE;
    extern int32 Legacy_GFX_FBUFFERMINUSONE;

    extern int32 Legacy_fadeMode;
    extern uint8 Legacy_fadeA;
    extern uint8 Legacy_fadeR;
    extern uint8 Legacy_fadeG;
    extern uint8 Legacy_fadeB;

    extern int32 Legacy_paletteMode;

    void Legacy_SetActivePalette(uint8 newActivePal, int32 startLine, int32 endLine);
    void Legacy_SetPaletteEntry(uint8 paletteIndex, uint8 index, uint8 r, uint8 g, uint8 b);
    void Legacy_SetPaletteEntryPacked(uint8 paletteIndex, uint8 index, uint32 color);
    uint32 Legacy_GetPaletteEntryPacked(uint8 bankID, uint8 index);
    void v3_CopyPalette(uint8 sourcePalette, uint8 destinationPalette);
    void v4_CopyPalette(uint8 sourcePalette, uint8 srcPaletteStart, uint8 destinationPalette, uint8 destPaletteStart, uint16 count);
    void v3_RotatePalette(uint8 startIndex, uint8 endIndex, bool right);
    void v4_RotatePalette(int32 palID, uint8 startIndex, uint8 endIndex, bool right);
    void Legacy_LoadPalette(const char *filePath, int32 paletteID, int32 startPaletteIndex, int32 startIndex, int32 endIndex);
    void Legacy_SetFade(uint8 R, uint8 G, uint8 B, uint16 A);
    void Legacy_SetPaletteFade(uint8 destPaletteID, uint8 srcPaletteA, uint8 srcPaletteB, uint16 blendAmount, int32 startIndex, int32 endIndex);
    void Legacy_SetLimitedFade(uint8 paletteID, uint8 R, uint8 G, uint8 B, uint16 blendAmount, int32 startIndex, int32 endIndex);
}

namespace Legacy
{
struct Color {
    uint8 r;
    uint8 g;
    uint8 b;
    uint8 a;
};
} // namespace Legacy