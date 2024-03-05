void RSDK::Legacy::LoadPalette(const char *filePath, int32 paletteID, int32 startPaletteIndex, int32 startIndex, int32 endIndex)
{
    char fullPath[0x80];

    StrCopy(fullPath, "Data/Palettes/");
    StrAdd(fullPath, filePath);

    FileInfo info;
    InitFileInfo(&info);

    if (LoadFile(&info, fullPath, FMODE_RB)) {
        Seek_Set(&info, 3 * startIndex);

        if (paletteID >= LEGACY_PALETTE_COUNT || paletteID < 0)
            paletteID = 0;

        uint8 color[3];
        if (paletteID) {
            for (int32 i = startIndex; i < endIndex; ++i) {
                ReadBytes(&info, &color, 3);
                SetPaletteEntry(paletteID, startPaletteIndex++, color[0], color[1], color[2]);
            }
        }
        else {
            for (int32 i = startIndex; i < endIndex; ++i) {
                ReadBytes(&info, &color, 3);
                SetPaletteEntry(-1, startPaletteIndex++, color[0], color[1], color[2]);
            }
        }

        CloseFile(&info);
    }
}

void RSDK::Legacy::SetPaletteFade(uint8 destPaletteID, uint8 srcPaletteA, uint8 srcPaletteB, uint16 blendAmount, int32 startIndex, int32 endIndex)
{
    if (destPaletteID >= LEGACY_PALETTE_COUNT || srcPaletteA >= LEGACY_PALETTE_COUNT || srcPaletteB >= LEGACY_PALETTE_COUNT)
        return;

    if (blendAmount >= 0x100)
        blendAmount = 0xFF;

    if (startIndex >= endIndex)
        return;

    uint32 blendA        = 0xFF - blendAmount;
    uint16 *paletteColor = &Legacy_fullPalette[destPaletteID][startIndex];
    for (int32 i = startIndex; i <= endIndex; ++i) {
        uint32 clrA = GetPaletteEntryPacked(srcPaletteA, i);
        uint32 clrB = GetPaletteEntryPacked(srcPaletteB, i);

        int32 r = blendAmount * ((clrB >> 0x10) & 0xFF) + blendA * ((clrA >> 0x10) & 0xFF);
        int32 g = blendAmount * ((clrB >> 0x08) & 0xFF) + blendA * ((clrA >> 0x08) & 0xFF);
        int32 b = blendAmount * ((clrB >> 0x00) & 0xFF) + blendA * ((clrA >> 0x00) & 0xFF);

        *paletteColor = PACK_RGB888((uint8)(r >> 8), (uint8)(g >> 8), (uint8)(b >> 8));

        ++paletteColor;
    }
}

void RSDK::Legacy::v3::SetLimitedFade(uint8 paletteID, uint8 R, uint8 G, uint8 B, uint16 blendAmount, int32 startIndex, int32 endIndex)
{
    if (paletteID >= LEGACY_PALETTE_COUNT)
        return;

    Legacy_paletteMode   = 1;
    Legacy_activePalette = Legacy_fullPalette[paletteID];

    if (blendAmount >= 0x100)
        blendAmount = 0xFF;

    if (startIndex >= endIndex)
        return;

    uint32 blendA        = 0xFF - blendAmount;
    uint16 *paletteColor = &Legacy_fullPalette[paletteID][startIndex];
    for (int32 i = startIndex; i <= endIndex; ++i) {
        uint32 clrA = GetPaletteEntryPacked(paletteID, i);

        int32 r = blendAmount * R + blendA * ((clrA >> 0x10) & 0xFF);
        int32 g = blendAmount * G + blendA * ((clrA >> 0x08) & 0xFF);
        int32 b = blendAmount * B + blendA * ((clrA >> 0x00) & 0xFF);

        *paletteColor = PACK_RGB888((uint8)(r >> 8), (uint8)(g >> 8), (uint8)(b >> 8));

        ++paletteColor;
    }
}