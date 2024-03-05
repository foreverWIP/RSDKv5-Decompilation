#define LEGACY_PALETTE_COUNT       (0x8)
#define LEGACY_PALETTE_COLOR_COUNT (0x100)

extern "C" {
    // Palettes (as RGB565 Colors)
    extern uint16 Legacy_fullPalette[LEGACY_PALETTE_COUNT][LEGACY_PALETTE_COLOR_COUNT];
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
}

namespace Legacy
{
struct Color {
    uint8 r;
    uint8 g;
    uint8 b;
    uint8 a;
};

void LoadPalette(const char *filePath, int32 paletteID, int32 startPaletteIndex, int32 startIndex, int32 endIndex);

inline void SetActivePalette(uint8 newActivePal, int32 startLine, int32 endLine)
{
    if (newActivePal < LEGACY_PALETTE_COUNT)
        for (int32 l = startLine; l < endLine && l < SCREEN_YSIZE; l++) Legacy_gfxLineBuffer[l] = newActivePal;

    Legacy_activePalette = Legacy_fullPalette[Legacy_gfxLineBuffer[0]];
}

inline void SetPaletteEntry(uint8 paletteIndex, uint8 index, uint8 r, uint8 g, uint8 b)
{
    if (paletteIndex != 0xFF) {
        Legacy_fullPalette[paletteIndex][index] = PACK_RGB888(r, g, b);
    }
    else {
        Legacy_activePalette[index] = PACK_RGB888(r, g, b);
    }
}

inline void SetPaletteEntryPacked(uint8 paletteIndex, uint8 index, uint32 color)
{
    Legacy_fullPalette[paletteIndex][index] = PACK_RGB888((uint8)(color >> 16), (uint8)(color >> 8), (uint8)(color >> 0));
}

inline uint32 GetPaletteEntryPacked(uint8 bankID, uint8 index)
{
    // 0xF800 = 1111 1000 0000 0000 = R
    // 0x7E0  = 0000 0111 1110 0000 = G
    // 0x1F   = 0000 0000 0001 1111 = B
    uint16 clr = Legacy_fullPalette[bankID & 7][index];

    int32 R = (clr & 0xF800) << 8;
    int32 G = (clr & 0x7E0) << 5;
    int32 B = (clr & 0x1F) << 3;
    return R | G | B;
}

inline void CopyPalette(uint8 sourcePalette, uint8 srcPaletteStart, uint8 destinationPalette, uint8 destPaletteStart, uint16 count)
{
    if (sourcePalette < LEGACY_PALETTE_COUNT && destinationPalette < LEGACY_PALETTE_COUNT) {
        for (int32 i = 0; i < count; ++i) {
            Legacy_fullPalette[destinationPalette][destPaletteStart + i] = Legacy_fullPalette[sourcePalette][srcPaletteStart + i];
        }
    }
}

inline void RotatePalette(int32 palID, uint8 startIndex, uint8 endIndex, bool right)
{
    if (right) {
        uint16 startClr = Legacy_fullPalette[palID][endIndex];
        for (int32 i = endIndex; i > startIndex; --i) {
            Legacy_fullPalette[palID][i] = Legacy_fullPalette[palID][i - 1];
        }
        Legacy_fullPalette[palID][startIndex] = startClr;
    }
    else {
        uint16 startClr = Legacy_fullPalette[palID][startIndex];
        for (int32 i = startIndex; i < endIndex; ++i) {
            Legacy_fullPalette[palID][i] = Legacy_fullPalette[palID][i + 1];
        }
        Legacy_fullPalette[palID][endIndex] = startClr;
    }
}

inline void SetFade(uint8 R, uint8 G, uint8 B, uint16 A)
{
    Legacy_fadeMode = 1;
    Legacy_fadeR    = R;
    Legacy_fadeG    = G;
    Legacy_fadeB    = B;
    Legacy_fadeA    = A > 0xFF ? 0xFF : A;
}

void SetPaletteFade(uint8 destPaletteID, uint8 srcPaletteA, uint8 srcPaletteB, uint16 blendAmount, int32 startIndex, int32 endIndex);

namespace v3
{

inline void CopyPalette(uint8 sourcePalette, uint8 destinationPalette)
{
    if (sourcePalette < LEGACY_PALETTE_COUNT && destinationPalette < LEGACY_PALETTE_COUNT) {
        for (int32 i = 0; i < LEGACY_PALETTE_COLOR_COUNT; ++i) {
            Legacy_fullPalette[destinationPalette][i] = Legacy_fullPalette[sourcePalette][i];
        }
    }
}

inline void RotatePalette(uint8 startIndex, uint8 endIndex, bool right)
{
    if (right) {
        uint16 startClr = Legacy_activePalette[endIndex];
        for (int32 i = endIndex; i > startIndex; --i) {
            Legacy_activePalette[i] = Legacy_activePalette[i - 1];
        }
        Legacy_activePalette[startIndex] = startClr;
    }
    else {
        uint16 startClr = Legacy_activePalette[startIndex];
        for (int32 i = startIndex; i < endIndex; ++i) {
            Legacy_activePalette[i] = Legacy_activePalette[i + 1];
        }
        Legacy_activePalette[endIndex] = startClr;
    }
}

void SetLimitedFade(uint8 paletteID, uint8 R, uint8 G, uint8 B, uint16 blendAmount, int32 startIndex, int32 endIndex);
} // namespace v3

} // namespace Legacy