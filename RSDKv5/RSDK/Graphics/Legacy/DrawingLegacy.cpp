

#include "RSDK/Graphics/Drawing.hpp"
#include "v3/DrawingLegacyv3.cpp"
#include "v4/DrawingLegacyv4.cpp"

int32 RSDK::Legacy::SCREEN_XSIZE = 424;
int32 RSDK::Legacy::SCREEN_CENTERX = 424 / 2;

RSDK::Legacy::DrawListEntry
    RSDK::Legacy::drawListEntries[LEGACY_DRAWLAYER_COUNT];

int32 RSDK::Legacy::gfxDataPosition;
RSDK::Legacy::GFXSurface RSDK::Legacy::gfxSurface[LEGACY_SURFACE_COUNT];
uint8 RSDK::Legacy::graphicData[LEGACY_GFXDATA_SIZE];

void RSDK::Legacy::ClearScreen(uint8 index) {
  uint16 color = Legacy_activePalette[index];
  uint16 *framebuffer = currentScreen->frameBuffer;

  int32 cnt = Legacy_GFX_LINESIZE * SCREEN_YSIZE;
  while (cnt--) {
    *framebuffer = color;
    ++framebuffer;
  }
}

void RSDK::Legacy::DrawHLineScrollLayer(int32 layerID) {
  TileLayer *layer = &stageLayouts[activeTileLayers[layerID]];
  if (!layer->xsize || !layer->ysize)
    return;

  int32 screenwidth16 = (Legacy_GFX_LINESIZE >> 4) - 1;
  int32 layerwidth = layer->xsize;
  int32 layerheight = layer->ysize;
  bool32 aboveMidPoint = layerID >= tLayerMidPoint;

  uint8 *lineScroll;
  int32 *deformationData;
  int32 *deformationDataW;

  int32 yscrollOffset = 0;
  if (activeTileLayers[layerID]) { // BG Layer
    int32 yScroll = yScrollOffset * layer->parallaxFactor >> 8;
    int32 fullheight = layerheight << 7;
    layer->scrollPos += layer->scrollSpeed;
    if (layer->scrollPos > fullheight << 16)
      layer->scrollPos -= fullheight << 16;
    yscrollOffset = (yScroll + (layer->scrollPos >> 16)) % fullheight;
    layerheight = fullheight >> 7;
    lineScroll = layer->lineScroll;
    deformationData =
        &bgDeformationData2[(uint8)(yscrollOffset + layer->deformationOffset)];
    deformationDataW = &bgDeformationData3[(
        uint8)(yscrollOffset + waterDrawPos + layer->deformationOffsetW)];
  } else { // FG Layer
    lastXSize = layer->xsize;
    yscrollOffset = yScrollOffset;
    lineScroll = layer->lineScroll;
    for (int32 i = 0; i < LEGACY_PARALLAX_COUNT; ++i)
      hParallax.linePos[i] = xScrollOffset;
    deformationData =
        &bgDeformationData0[(uint8)(yscrollOffset + layer->deformationOffset)];
    deformationDataW = &bgDeformationData1[(
        uint8)(yscrollOffset + waterDrawPos + layer->deformationOffsetW)];
  }

  if (layer->type == LAYER_HSCROLL) {
    if (lastXSize != layerwidth) {
      int32 fullLayerwidth = layerwidth << 7;
      for (int32 i = 0; i < hParallax.entryCount; ++i) {
        hParallax.linePos[i] = xScrollOffset * hParallax.parallaxFactor[i] >> 8;
        if (hParallax.scrollPos[i] > fullLayerwidth << 16)
          hParallax.scrollPos[i] -= fullLayerwidth << 16;
        if (hParallax.scrollPos[i] < 0)
          hParallax.scrollPos[i] += fullLayerwidth << 16;
        hParallax.linePos[i] += hParallax.scrollPos[i] >> 16;
        hParallax.linePos[i] %= fullLayerwidth;
      }
    }
    int32 w = -1;
    if (activeTileLayers[layerID])
      w = layerwidth;
    lastXSize = w;
  }

  uint16 *frameBuffer = currentScreen->frameBuffer;
  uint8 *lineBuffer = gfxLineBuffer;
  int32 tileYPos = yscrollOffset % (layerheight << 7);
  if (tileYPos < 0)
    tileYPos += layerheight << 7;
  uint8 *scrollIndex = &lineScroll[tileYPos];
  int32 tileY16 = tileYPos & 0xF;
  int32 chunkY = tileYPos >> 7;
  int32 tileY = (tileYPos & 0x7F) >> 4;

  // Draw Above Water (if applicable)
  int32 drawableLines[2] = {waterDrawPos, SCREEN_YSIZE - waterDrawPos};
  for (int32 i = 0; i < 2; ++i) {
    while (drawableLines[i]--) {
      Legacy_activePalette = fullPalette[*lineBuffer];
      lineBuffer++;

      int32 chunkX = hParallax.linePos[*scrollIndex];
      if (i == 0) {
        if (hParallax.deform[*scrollIndex])
          chunkX += *deformationData;
        ++deformationData;
      } else {
        if (hParallax.deform[*scrollIndex])
          chunkX += *deformationDataW;
        ++deformationDataW;
      }
      ++scrollIndex;

      int32 fullLayerwidth = layerwidth << 7;
      if (chunkX < 0)
        chunkX += fullLayerwidth;
      if (chunkX >= fullLayerwidth)
        chunkX -= fullLayerwidth;

      int32 chunkXPos = chunkX >> 7;
      int32 tilePxXPos = chunkX & 0xF;
      int32 tileXPxRemain = TILE_SIZE - tilePxXPos;
      int32 chunk = (layer->tiles[(chunkX >> 7) + (chunkY << 8)] << 6) +
                    ((chunkX & 0x7F) >> 4) + 8 * tileY;
      int32 tileOffsetY = TILE_SIZE * tileY16;
      int32 tileOffsetYFlipX = TILE_SIZE * tileY16 + 0xF;
      int32 tileOffsetYFlipY = TILE_SIZE * (0xF - tileY16);
      int32 tileOffsetYFlipXY = TILE_SIZE * (0xF - tileY16) + 0xF;
      int32 lineRemain = Legacy_GFX_LINESIZE;

      uint8 *pixels = NULL;
      int32 tilePxLineCnt = tileXPxRemain;

      // Draw the first tile to the left
      if (tiles128x128.visualPlane[chunk] == (uint8)aboveMidPoint) {
        lineRemain -= tilePxLineCnt;
        switch (tiles128x128.direction[chunk]) {
        case FLIP_NONE:
          pixels = &tilesetGFXData[tileOffsetY +
                                   tiles128x128.gfxDataPos[chunk] + tilePxXPos];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;
          }
          break;

        case FLIP_X:

          pixels = &tilesetGFXData[tileOffsetYFlipX +
                                   tiles128x128.gfxDataPos[chunk] - tilePxXPos];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;
          }
          break;

        case FLIP_Y:
          pixels = &tilesetGFXData[tileOffsetYFlipY +
                                   tiles128x128.gfxDataPos[chunk] + tilePxXPos];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;
          }
          break;

        case FLIP_XY:
          pixels = &tilesetGFXData[tileOffsetYFlipXY +
                                   tiles128x128.gfxDataPos[chunk] - tilePxXPos];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;
          }
          break;

        default:
          break;
        }
      } else {
        frameBuffer += tilePxLineCnt;
        lineRemain -= tilePxLineCnt;
      }

      // Draw the bulk of the tiles
      int32 chunkTileX = ((chunkX & 0x7F) >> 4) + 1;
      int32 tilesPerLine = screenwidth16;
      while (tilesPerLine--) {
        if (chunkTileX < 8) {
          ++chunk;
        } else {
          if (++chunkXPos == layerwidth)
            chunkXPos = 0;

          chunkTileX = 0;
          chunk = (layer->tiles[chunkXPos + (chunkY << 8)] << 6) + 8 * tileY;
        }
        lineRemain -= TILE_SIZE;

        // Loop Unrolling (faster but messier code)
        if (tiles128x128.visualPlane[chunk] == (uint8)aboveMidPoint) {
          switch (tiles128x128.direction[chunk]) {
          case FLIP_NONE:
            pixels =
                &tilesetGFXData[tiles128x128.gfxDataPos[chunk] + tileOffsetY];
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            break;

          case FLIP_X:
            pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                     tileOffsetYFlipX];
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            break;

          case FLIP_Y:
            pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                     tileOffsetYFlipY];
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            ++pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            break;

          case FLIP_XY:
            pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                     tileOffsetYFlipXY];
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            --pixels;

            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            ++frameBuffer;
            break;
          }
        } else {
          frameBuffer += TILE_SIZE;
        }
        ++chunkTileX;
      }

      // Draw any remaining tiles
      while (lineRemain > 0) {
        if (chunkTileX++ < 8) {
          ++chunk;
        } else {
          chunkTileX = 0;
          if (++chunkXPos == layerwidth)
            chunkXPos = 0;

          chunk = (layer->tiles[chunkXPos + (chunkY << 8)] << 6) + 8 * tileY;
        }

        tilePxLineCnt = lineRemain >= TILE_SIZE ? TILE_SIZE : lineRemain;
        lineRemain -= tilePxLineCnt;
        if (tiles128x128.visualPlane[chunk] == (uint8)aboveMidPoint) {
          switch (tiles128x128.direction[chunk]) {
          case FLIP_NONE:
            pixels =
                &tilesetGFXData[tiles128x128.gfxDataPos[chunk] + tileOffsetY];
            while (tilePxLineCnt--) {
              if (*pixels > 0)
                *frameBuffer = Legacy_activePalette[*pixels];
              ++frameBuffer;
              ++pixels;
            }
            break;

          case FLIP_X:
            pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                     tileOffsetYFlipX];
            while (tilePxLineCnt--) {
              if (*pixels > 0)
                *frameBuffer = Legacy_activePalette[*pixels];
              ++frameBuffer;
              --pixels;
            }
            break;

          case FLIP_Y:
            pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                     tileOffsetYFlipY];
            while (tilePxLineCnt--) {
              if (*pixels > 0)
                *frameBuffer = Legacy_activePalette[*pixels];
              ++frameBuffer;
              ++pixels;
            }
            break;

          case FLIP_XY:
            pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                     tileOffsetYFlipXY];
            while (tilePxLineCnt--) {
              if (*pixels > 0)
                *frameBuffer = Legacy_activePalette[*pixels];
              ++frameBuffer;
              --pixels;
            }
            break;

          default:
            break;
          }
        } else {
          frameBuffer += tilePxLineCnt;
        }
      }

      if (++tileY16 >= TILE_SIZE) {
        tileY16 = 0;
        ++tileY;
      }

      if (tileY >= 8) {
        if (++chunkY == layerheight) {
          chunkY = 0;
          scrollIndex -= 0x80 * layerheight;
        }
        tileY = 0;
      }
    }
  }
}
void RSDK::Legacy::DrawVLineScrollLayer(int32 layerID) {
  TileLayer *layer = &stageLayouts[activeTileLayers[layerID]];
  if (!layer->xsize || !layer->ysize)
    return;

  int32 layerwidth = layer->xsize;
  int32 layerheight = layer->ysize;
  bool aboveMidPoint = layerID >= tLayerMidPoint;

  uint8 *lineScroll;
  int32 *deformationData;

  int32 xscrollOffset = 0;
  if (activeTileLayers[layerID]) { // BG Layer
    int32 xScroll = xScrollOffset * layer->parallaxFactor >> 8;
    int32 fullLayerwidth = layerwidth << 7;
    layer->scrollPos += layer->scrollSpeed;
    if (layer->scrollPos > fullLayerwidth << 16)
      layer->scrollPos -= fullLayerwidth << 16;
    xscrollOffset = (xScroll + (layer->scrollPos >> 16)) % fullLayerwidth;
    layerwidth = fullLayerwidth >> 7;
    lineScroll = layer->lineScroll;
    deformationData =
        &bgDeformationData2[(uint8)(xscrollOffset + layer->deformationOffset)];
  } else { // FG Layer
    lastYSize = layer->ysize;
    xscrollOffset = xScrollOffset;
    lineScroll = layer->lineScroll;
    vParallax.linePos[0] = yScrollOffset;
    vParallax.deform[0] = true;
    deformationData =
        &bgDeformationData0[(uint8)(xScrollOffset + layer->deformationOffset)];
  }

  if (layer->type == LAYER_VSCROLL) {
    if (lastYSize != layerheight) {
      int32 fullLayerheight = layerheight << 7;
      for (int32 i = 0; i < vParallax.entryCount; ++i) {
        vParallax.linePos[i] = yScrollOffset * vParallax.parallaxFactor[i] >> 8;

        vParallax.scrollPos[i] += vParallax.scrollPos[i] << 16;
        if (vParallax.scrollPos[i] > fullLayerheight << 16)
          vParallax.scrollPos[i] -= fullLayerheight << 16;

        vParallax.linePos[i] += vParallax.scrollPos[i] >> 16;
        vParallax.linePos[i] %= fullLayerheight;
      }
      layerheight = fullLayerheight >> 7;
    }
    lastYSize = layerheight;
  }

  uint16 *frameBuffer = currentScreen->frameBuffer;
  Legacy_activePalette = fullPalette[gfxLineBuffer[0]];
  int32 tileXPos = xscrollOffset % (layerheight << 7);
  if (tileXPos < 0)
    tileXPos += layerheight << 7;
  uint8 *scrollIndex = &lineScroll[tileXPos];
  int32 chunkX = tileXPos >> 7;
  int32 tileX16 = tileXPos & 0xF;
  int32 tileX = (tileXPos & 0x7F) >> 4;

  // Draw Above Water (if applicable)
  int32 drawableLines = SCREEN_XSIZE;
  while (drawableLines--) {
    int32 chunkY = vParallax.linePos[*scrollIndex];
    if (vParallax.deform[*scrollIndex])
      chunkY += *deformationData;
    ++deformationData;
    ++scrollIndex;

    int32 fullLayerHeight = layerheight << 7;
    if (chunkY < 0)
      chunkY += fullLayerHeight;
    if (chunkY >= fullLayerHeight)
      chunkY -= fullLayerHeight;

    int32 chunkYPos = chunkY >> 7;
    int32 tileY = chunkY & 0xF;
    int32 tileYPxRemain = TILE_SIZE - tileY;
    int32 chunk = (layer->tiles[chunkX + (chunkY >> 7 << 8)] << 6) + tileX +
                  8 * ((chunkY & 0x7F) >> 4);
    int32 tileOffsetXFlipX = 0xF - tileX16;
    int32 tileOffsetXFlipY = tileX16 + SCREEN_YSIZE;
    int32 tileOffsetXFlipXY = 0xFF - tileX16;
    int32 lineRemain = SCREEN_YSIZE;

    uint8 *pixels = NULL;
    int32 tilePxLineCnt = tileYPxRemain;

    // Draw the first tile to the left
    if (tiles128x128.visualPlane[chunk] == (uint8)aboveMidPoint) {
      lineRemain -= tilePxLineCnt;
      switch (tiles128x128.direction[chunk]) {
      case FLIP_NONE:
        pixels = &tilesetGFXData[TILE_SIZE * tileY + tileX16 +
                                 tiles128x128.gfxDataPos[chunk]];
        while (tilePxLineCnt--) {
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;
        }
        break;

      case FLIP_X:
        pixels = &tilesetGFXData[TILE_SIZE * tileY + tileOffsetXFlipX +
                                 tiles128x128.gfxDataPos[chunk]];
        while (tilePxLineCnt--) {
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;
        }
        break;

      case FLIP_Y:
        pixels =
            &tilesetGFXData[tileOffsetXFlipY + tiles128x128.gfxDataPos[chunk] -
                            TILE_SIZE * tileY];
        while (tilePxLineCnt--) {
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;
        }
        break;

      case FLIP_XY:
        pixels =
            &tilesetGFXData[tileOffsetXFlipXY + tiles128x128.gfxDataPos[chunk] -
                            TILE_SIZE * tileY];
        while (tilePxLineCnt--) {
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;
        }
        break;

      default:
        break;
      }
    } else {
      frameBuffer += Legacy_GFX_LINESIZE * tileYPxRemain;
      lineRemain -= tilePxLineCnt;
    }

    // Draw the bulk of the tiles
    int32 chunkTileY = ((chunkY & 0x7F) >> 4) + 1;
    int32 tilesPerLine = (SCREEN_YSIZE >> 4) - 1;

    while (tilesPerLine--) {
      if (chunkTileY < 8) {
        chunk += 8;
      } else {
        if (++chunkYPos == layerheight)
          chunkYPos = 0;

        chunkTileY = 0;
        chunk = (layer->tiles[chunkX + (chunkYPos << 8)] << 6) + tileX;
      }
      lineRemain -= TILE_SIZE;

      // Loop Unrolling (faster but messier code)
      if (tiles128x128.visualPlane[chunk] == (uint8)aboveMidPoint) {
        switch (tiles128x128.direction[chunk]) {
        case FLIP_NONE:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] + tileX16];
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          break;

        case FLIP_X:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                   tileOffsetXFlipX];
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels += TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          break;

        case FLIP_Y:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                   tileOffsetXFlipY];
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          break;

        case FLIP_XY:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                   tileOffsetXFlipXY];
          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          pixels -= TILE_SIZE;

          if (*pixels > 0)
            *frameBuffer = Legacy_activePalette[*pixels];
          frameBuffer += Legacy_GFX_LINESIZE;
          break;
        }
      } else {
        frameBuffer += Legacy_GFX_LINESIZE * TILE_SIZE;
      }
      ++chunkTileY;
    }

    // Draw any remaining tiles
    while (lineRemain > 0) {
      if (chunkTileY < 8) {
        chunk += 8;
      } else {
        if (++chunkYPos == layerheight)
          chunkYPos = 0;

        chunkTileY = 0;
        chunk = (layer->tiles[chunkX + (chunkYPos << 8)] << 6) + tileX;
      }

      tilePxLineCnt = lineRemain >= TILE_SIZE ? TILE_SIZE : lineRemain;
      lineRemain -= tilePxLineCnt;

      if (tiles128x128.visualPlane[chunk] == (uint8)aboveMidPoint) {
        switch (tiles128x128.direction[chunk]) {
        case FLIP_NONE:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] + tileX16];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            frameBuffer += Legacy_GFX_LINESIZE;
            pixels += TILE_SIZE;
          }
          break;

        case FLIP_X:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                   tileOffsetXFlipX];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            frameBuffer += Legacy_GFX_LINESIZE;
            pixels += TILE_SIZE;
          }
          break;

        case FLIP_Y:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                   tileOffsetXFlipY];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            frameBuffer += Legacy_GFX_LINESIZE;
            pixels -= TILE_SIZE;
          }
          break;

        case FLIP_XY:
          pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk] +
                                   tileOffsetXFlipXY];
          while (tilePxLineCnt--) {
            if (*pixels > 0)
              *frameBuffer = Legacy_activePalette[*pixels];
            frameBuffer += Legacy_GFX_LINESIZE;
            pixels -= TILE_SIZE;
          }
          break;

        default:
          break;
        }
      } else {
        frameBuffer += Legacy_GFX_LINESIZE * tilePxLineCnt;
      }
      chunkTileY++;
    }

    if (++tileX16 >= TILE_SIZE) {
      tileX16 = 0;
      ++tileX;
    }

    if (tileX >= 8) {
      if (++chunkX == layerwidth) {
        chunkX = 0;
        scrollIndex -= 0x80 * layerwidth;
      }
      tileX = 0;
    }

    frameBuffer -= Legacy_GFX_FBUFFERMINUSONE;
  }
}
void RSDK::Legacy::Draw3DFloorLayer(int32 layerID) {
  TileLayer *layer = &stageLayouts[activeTileLayers[layerID]];
  if (!layer->xsize || !layer->ysize)
    return;

  int32 layerWidth = layer->xsize << 7;
  int32 layerHeight = layer->ysize << 7;
  int32 layerYPos = layer->ypos;
  int32 layerZPos = layer->zpos;
  int32 sinValue = sinM7LookupTable[layer->angle];
  int32 cosValue = cosM7LookupTable[layer->angle];
  uint8 *lineBuffer = &gfxLineBuffer[(SCREEN_YSIZE / 2) + 12];
  uint16 *frameBuffer =
      &currentScreen
           ->frameBuffer[((SCREEN_YSIZE / 2) + 12) * Legacy_GFX_LINESIZE];
  int32 layerXPos = layer->xpos >> 4;
  int32 ZBuffer = layerZPos >> 4;

  for (int32 i = 4; i < 112; ++i) {
    if (!(i & 1)) {
      Legacy_activePalette = fullPalette[*lineBuffer];
      lineBuffer++;
    }
    int32 XBuffer = layerYPos / (i << 9) * -cosValue >> 8;
    int32 YBuffer = sinValue * (layerYPos / (i << 9)) >> 8;
    int32 XPos = layerXPos + (3 * sinValue * (layerYPos / (i << 9)) >> 2) -
                 XBuffer * SCREEN_CENTERX;
    int32 YPos = ZBuffer + (3 * cosValue * (layerYPos / (i << 9)) >> 2) -
                 YBuffer * SCREEN_CENTERX;
    int32 lineBuffer = 0;
    while (lineBuffer < Legacy_GFX_LINESIZE) {
      int32 tileX = XPos >> 12;
      int32 tileY = YPos >> 12;
      if (tileX > -1 && tileX < layerWidth && tileY > -1 &&
          tileY < layerHeight) {
        int32 chunk = tile3DFloorBuffer[(YPos >> 16 << 8) + (XPos >> 16)];
        uint8 *pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk]];
        switch (tiles128x128.direction[chunk]) {
        case FLIP_NONE:
          pixels += TILE_SIZE * (tileY & 0xF) + (tileX & 0xF);
          break;
        case FLIP_X:
          pixels += TILE_SIZE * (tileY & 0xF) + 15 - (tileX & 0xF);
          break;
        case FLIP_Y:
          pixels += (tileX & 0xF) + SCREEN_YSIZE - TILE_SIZE * (tileY & 0xF);
          break;
        case FLIP_XY:
          pixels +=
              15 - (tileX & 0xF) + SCREEN_YSIZE - TILE_SIZE * (tileY & 0xF);
          break;
        default:
          break;
        }

        if (*pixels > 0)
          *frameBuffer = Legacy_activePalette[*pixels];
      }
      ++frameBuffer;
      ++lineBuffer;

      XPos += XBuffer;
      YPos += YBuffer;
    }
  }
}
void RSDK::Legacy::Draw3DSkyLayer(int32 layerID) {
  TileLayer *layer = &stageLayouts[activeTileLayers[layerID]];
  if (!layer->xsize || !layer->ysize)
    return;

  int32 layerWidth = layer->xsize << 7;
  int32 layerHeight = layer->ysize << 7;
  int32 layerYPos = layer->ypos;
  int32 sinValue = sinM7LookupTable[layer->angle & 0x1FF];
  int32 cosValue = cosM7LookupTable[layer->angle & 0x1FF];
  uint16 *frameBuffer =
      &currentScreen
           ->frameBuffer[((SCREEN_YSIZE / 2) + 12) * Legacy_GFX_LINESIZE];
  uint8 *lineBuffer = &gfxLineBuffer[((SCREEN_YSIZE / 2) + 12)];
  int32 layerXPos = layer->xpos >> 4;
  int32 layerZPos = layer->zpos >> 4;
  for (int32 i = TILE_SIZE / 2; i < SCREEN_YSIZE - TILE_SIZE; ++i) {
    if (!(i & 1)) {
      Legacy_activePalette = fullPalette[*lineBuffer];
      lineBuffer++;
    }

    int32 xBuffer = layerYPos / (i << 8) * -cosValue >> 9;
    int32 yBuffer = sinValue * (layerYPos / (i << 8)) >> 9;
    int32 XPos = layerXPos + (3 * sinValue * (layerYPos / (i << 8)) >> 2) -
                 xBuffer * Legacy_GFX_LINESIZE;
    int32 YPos = layerZPos + (3 * cosValue * (layerYPos / (i << 8)) >> 2) -
                 yBuffer * Legacy_GFX_LINESIZE;
    int32 lineBuffer = 0;

    while (lineBuffer < Legacy_GFX_LINESIZE * 2) {
      int32 tileX = XPos >> 12;
      int32 tileY = YPos >> 12;
      if (tileX > -1 && tileX < layerWidth && tileY > -1 &&
          tileY < layerHeight) {
        int32 chunk = tile3DFloorBuffer[(YPos >> 16 << 8) + (XPos >> 16)];
        uint8 *pixels = &tilesetGFXData[tiles128x128.gfxDataPos[chunk]];

        switch (tiles128x128.direction[chunk]) {
        case FLIP_NONE:
          pixels += TILE_SIZE * (tileY & 0xF) + (tileX & 0xF);
          break;
        case FLIP_X:
          pixels += TILE_SIZE * (tileY & 0xF) + 0xF - (tileX & 0xF);
          break;
        case FLIP_Y:
          pixels += (tileX & 0xF) + SCREEN_YSIZE - TILE_SIZE * (tileY & 0xF);
          break;
        case FLIP_XY:
          pixels +=
              0xF - (tileX & 0xF) + SCREEN_YSIZE - TILE_SIZE * (tileY & 0xF);
          break;
        default:
          break;
        }

        if (*pixels > 0)
          *frameBuffer = Legacy_activePalette[*pixels];
      }

      if (lineBuffer & 1)
        ++frameBuffer;

      lineBuffer++;
      XPos += xBuffer;
      YPos += yBuffer;
    }

    if (!(i & 1))
      frameBuffer -= Legacy_GFX_LINESIZE;
  }
}

void RSDK::Legacy::DrawRectangle(int32 XPos, int32 YPos, int32 width,
                                 int32 height, int32 R, int32 G, int32 B,
                                 int32 A) {
  int32 inkEffect = INK_ALPHA;
  if (A >= 0xFF) {
    inkEffect = INK_NONE;
  }
  DrawRectangle(XPos, YPos, width, height, (R << 16) | (G << 8) | B, A, inkEffect, true);
}

void RSDK::Legacy::DrawTintRectangle(int32 XPos, int32 YPos, int32 width,
                                     int32 height) {
  DrawRectangle(XPos, YPos, width, height, 0, 0, INK_TINT, true);
}
void RSDK::Legacy::DrawScaledTintMask(int32 direction, int32 XPos, int32 YPos,
                                      int32 pivotX, int32 pivotY, int32 scaleX,
                                      int32 scaleY, int32 width, int32 height,
                                      int32 sprX, int32 sprY, int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  DrawSpriteRotozoomGeneric(XPos, YPos, -pivotX, -pivotY, width, height, sprX, sprY, scaleX, scaleY, direction, 0, INK_TINT, 0xff, &graphicData[surface->dataPosition], surface->widthShift);
}

void RSDK::Legacy::DrawSprite(int32 XPos, int32 YPos, int32 width, int32 height,
                              int32 sprX, int32 sprY, int32 sheetID) {
  RSDK::Legacy::DrawSpriteFlipped(XPos, YPos, width, height, sprX, sprY, FLIP_NONE, sheetID);
}

void RSDK::Legacy::DrawSpriteFlipped(int32 XPos, int32 YPos, int32 width,
                                     int32 height, int32 sprX, int32 sprY,
                                     int32 direction, int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  RSDK::DrawSpriteFlippedGeneric(XPos, YPos, width, height, sprX, sprY,
                                 direction, INK_NONE, 0xff, surface->width,
                                 &graphicData[surface->dataPosition]);
}
void RSDK::Legacy::DrawSpriteScaled(int32 direction, int32 XPos, int32 YPos,
                                    int32 pivotX, int32 pivotY, int32 scaleX,
                                    int32 scaleY, int32 width, int32 height,
                                    int32 sprX, int32 sprY, int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  DrawSpriteRotozoomGeneric(XPos, YPos, -pivotX, -pivotY, width, height, sprX, sprY, scaleX, scaleY, direction, 0, INK_NONE, 0xff, &graphicData[surface->dataPosition], surface->widthShift);
}
void RSDK::Legacy::DrawSpriteRotated(int32 direction, int32 XPos, int32 YPos,
                                     int32 pivotX, int32 pivotY, int32 sprX,
                                     int32 sprY, int32 width, int32 height,
                                     int32 rotation, int32 sheetID) {
  RSDK::Legacy::DrawSpriteRotozoom(direction, XPos, YPos, pivotX, pivotY, sprX, sprY, width, height, rotation, 0x200, sheetID);
}

void RSDK::Legacy::DrawSpriteRotozoom(int32 direction, int32 XPos, int32 YPos,
                                      int32 pivotX, int32 pivotY, int32 sprX,
                                      int32 sprY, int32 width, int32 height,
                                      int32 rotation, int32 scale,
                                      int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  DrawSpriteRotozoomGeneric(XPos, YPos, -pivotX, -pivotY, width, height, sprX, sprY, scale, scale, direction, rotation, INK_NONE, 0xff, &graphicData[surface->dataPosition], surface->widthShift);
}

void RSDK::Legacy::DrawBlendedSprite(int32 XPos, int32 YPos, int32 width,
                                     int32 height, int32 sprX, int32 sprY,
                                     int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  RSDK::DrawSpriteFlippedGeneric(XPos, YPos, width, height, sprX, sprY,
                                 FLIP_NONE, INK_BLEND, 0xff, surface->width,
                                 &graphicData[surface->dataPosition]);
}
void RSDK::Legacy::DrawAlphaBlendedSprite(int32 XPos, int32 YPos, int32 width,
                                          int32 height, int32 sprX, int32 sprY,
                                          int32 alpha, int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  RSDK::DrawSpriteFlippedGeneric(XPos, YPos, width, height, sprX, sprY,
                                 FLIP_NONE, INK_ALPHA, alpha, surface->width,
                                 &graphicData[surface->dataPosition]);
}
void RSDK::Legacy::DrawAdditiveBlendedSprite(int32 XPos, int32 YPos,
                                             int32 width, int32 height,
                                             int32 sprX, int32 sprY,
                                             int32 alpha, int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  RSDK::DrawSpriteFlippedGeneric(XPos, YPos, width, height, sprX, sprY,
                                 FLIP_NONE, INK_ADD, alpha, surface->width,
                                 &graphicData[surface->dataPosition]);
}
void RSDK::Legacy::DrawSubtractiveBlendedSprite(int32 XPos, int32 YPos,
                                                int32 width, int32 height,
                                                int32 sprX, int32 sprY,
                                                int32 alpha, int32 sheetID) {
  GFXSurface *surface = &gfxSurface[sheetID];
  RSDK::DrawSpriteFlippedGeneric(XPos, YPos, width, height, sprX, sprY,
                                 FLIP_NONE, INK_SUB, alpha, surface->width,
                                 &graphicData[surface->dataPosition]);
}

void RSDK::Legacy::DrawFace(void *v, uint32 color) {
  Vertex *verts = (Vertex *)v;

  if (verts[0].x < 0 && verts[1].x < 0 && verts[2].x < 0 && verts[3].x < 0)
      return;

  if (verts[0].x > Legacy_GFX_LINESIZE && verts[1].x > Legacy_GFX_LINESIZE && verts[2].x > Legacy_GFX_LINESIZE && verts[3].x > Legacy_GFX_LINESIZE)
      return;

  if (verts[0].y < 0 && verts[1].y < 0 && verts[2].y < 0 && verts[3].y < 0)
      return;

  if (verts[0].y > SCREEN_YSIZE && verts[1].y > SCREEN_YSIZE && verts[2].y > SCREEN_YSIZE && verts[3].y > SCREEN_YSIZE)
      return;

  if (verts[0].x == verts[1].x && verts[1].x == verts[2].x && verts[2].x == verts[3].x)
      return;

  if (verts[0].y == verts[1].y && verts[1].y == verts[2].y && verts[2].y == verts[3].y)
      return;

  Vector2 newVertices[4];
  newVertices[0] = { TO_FIXED(verts[0].x), TO_FIXED(verts[0].y) };
  newVertices[1] = { TO_FIXED(verts[1].x), TO_FIXED(verts[1].y) };
  newVertices[2] = { TO_FIXED(verts[2].x), TO_FIXED(verts[2].y) };
  newVertices[3] = { TO_FIXED(verts[3].x), TO_FIXED(verts[3].y) };

  RSDK::DrawFace((Vector2 *)&newVertices, 4, (color >> 16) & 0xFF, (color >> 8) & 0xFF, color & 0xFF, (color & 0x7F000000) >> 23, INK_ALPHA);
}
void RSDK::Legacy::DrawTexturedFace(void *v, uint8 sheetID) {
  Vertex *verts = (Vertex *)v;

  if (verts[0].x < 0 && verts[1].x < 0 && verts[2].x < 0 && verts[3].x < 0)
    return;
  if (verts[0].x > Legacy_GFX_LINESIZE && verts[1].x > Legacy_GFX_LINESIZE &&
      verts[2].x > Legacy_GFX_LINESIZE && verts[3].x > Legacy_GFX_LINESIZE)
    return;
  if (verts[0].y < 0 && verts[1].y < 0 && verts[2].y < 0 && verts[3].y < 0)
    return;
  if (verts[0].y > SCREEN_YSIZE && verts[1].y > SCREEN_YSIZE &&
      verts[2].y > SCREEN_YSIZE && verts[3].y > SCREEN_YSIZE)
    return;
  if (verts[0].x == verts[1].x && verts[1].x == verts[2].x &&
      verts[2].x == verts[3].x)
    return;
  if (verts[0].y == verts[1].y && verts[1].y == verts[2].y &&
      verts[2].y == verts[3].y)
    return;

  int32 vertexA = 0;
  int32 vertexB = 1;
  int32 vertexC = 2;
  int32 vertexD = 3;
  if (verts[1].y < verts[0].y) {
    vertexA = 1;
    vertexB = 0;
  }
  if (verts[2].y < verts[vertexA].y) {
    int32 temp = vertexA;
    vertexA = 2;
    vertexC = temp;
  }
  if (verts[3].y < verts[vertexA].y) {
    int32 temp = vertexA;
    vertexA = 3;
    vertexD = temp;
  }
  if (verts[vertexC].y < verts[vertexB].y) {
    int32 temp = vertexB;
    vertexB = vertexC;
    vertexC = temp;
  }
  if (verts[vertexD].y < verts[vertexB].y) {
    int32 temp = vertexB;
    vertexB = vertexD;
    vertexD = temp;
  }
  if (verts[vertexD].y < verts[vertexC].y) {
    int32 temp = vertexC;
    vertexC = vertexD;
    vertexD = temp;
  }

  int32 faceTop = verts[vertexA].y;
  int32 faceBottom = verts[vertexD].y;
  if (faceTop < 0)
    faceTop = 0;
  if (faceBottom > SCREEN_YSIZE)
    faceBottom = SCREEN_YSIZE;
  for (int32 i = faceTop; i < faceBottom; ++i) {
    RSDK::scanEdgeBuffer[i].start = 100000;
    RSDK::scanEdgeBuffer[i].end = -100000;
  }

  ProcessScanEdgeUV(&verts[vertexA], &verts[vertexB]);
  ProcessScanEdgeUV(&verts[vertexA], &verts[vertexC]);
  ProcessScanEdgeUV(&verts[vertexA], &verts[vertexD]);
  ProcessScanEdgeUV(&verts[vertexB], &verts[vertexC]);
  ProcessScanEdgeUV(&verts[vertexC], &verts[vertexD]);
  ProcessScanEdgeUV(&verts[vertexB], &verts[vertexD]);

  uint16 *frameBuffer =
      &currentScreen->frameBuffer[Legacy_GFX_LINESIZE * faceTop];
  uint8 *pixels = &graphicData[gfxSurface[sheetID].dataPosition];
  int32 shiftwidth = gfxSurface[sheetID].widthShift;
  uint8 *lineBuffer = &gfxLineBuffer[faceTop];

  while (faceTop < faceBottom) {
    Legacy_activePalette = fullPalette[*lineBuffer];
    lineBuffer++;

    int32 startX = RSDK::scanEdgeBuffer[faceTop].start;
    int32 endX = RSDK::scanEdgeBuffer[faceTop].end;
    int32 UPos = scanEdgeBufferU[faceTop].start;
    int32 VPos = scanEdgeBufferV[faceTop].start;
    if (startX >= Legacy_GFX_LINESIZE || endX <= 0) {
      frameBuffer += Legacy_GFX_LINESIZE;
    } else {
      int32 posDifference = endX - startX;
      int32 bufferedUPos = 0;
      int32 bufferedVPos = 0;
      if (endX == startX) {
        bufferedUPos = 0;
        bufferedVPos = 0;
      } else {
        bufferedUPos = (scanEdgeBufferU[faceTop].end - UPos) / posDifference;
        bufferedVPos = (scanEdgeBufferV[faceTop].end - VPos) / posDifference;
      }
      if (endX > Legacy_GFX_LINESIZE_MINUSONE)
        posDifference = Legacy_GFX_LINESIZE_MINUSONE - startX;
      if (startX < 0) {
        posDifference += startX;
        UPos -= startX * bufferedUPos;
        VPos -= startX * bufferedVPos;
        startX = 0;
      }

      uint16 *framebufferPtr = &frameBuffer[startX];
      frameBuffer += Legacy_GFX_LINESIZE;

      int32 counter = posDifference + 1;
      while (counter--) {
        if (UPos < 0)
          UPos = 0;
        if (VPos < 0)
          VPos = 0;
        uint16 index = pixels[(VPos >> 16 << shiftwidth) + (UPos >> 16)];
        if (index > 0)
          *framebufferPtr = Legacy_activePalette[index];
        framebufferPtr++;
        UPos += bufferedUPos;
        VPos += bufferedVPos;
      }
    }
    ++faceTop;
  }
}

void RSDK::Legacy::v4::DrawObjectAnimation(void *objScr, void *ent, int32 XPos,
                                           int32 YPos) {
  ObjectScript *objectScript = (ObjectScript *)objScr;
  Entity *entity = (Entity *)ent;
  SpriteAnimation *sprAnim =
      &Legacy_animationList[objectScript->animFile->aniListOffset + entity->animation];
  SpriteFrame *frame = &Legacy_animFrames[sprAnim->frameListOffset + entity->frame];
  int32 rotation = 0;

  switch (sprAnim->rotationStyle) {
  case ROTSTYLE_NONE:
    switch (entity->direction) {
    case FLIP_NONE:
      DrawSpriteFlipped(frame->pivotX + XPos, frame->pivotY + YPos,
                        frame->width, frame->height, frame->sprX, frame->sprY,
                        FLIP_NONE, frame->sheetID);
      break;

    case FLIP_X:
      DrawSpriteFlipped(XPos - frame->width - frame->pivotX,
                        frame->pivotY + YPos, frame->width, frame->height,
                        frame->sprX, frame->sprY, FLIP_X, frame->sheetID);
      break;
    case FLIP_Y:

      DrawSpriteFlipped(frame->pivotX + XPos,
                        YPos - frame->height - frame->pivotY, frame->width,
                        frame->height, frame->sprX, frame->sprY, FLIP_Y,
                        frame->sheetID);
      break;

    case FLIP_XY:
      DrawSpriteFlipped(XPos - frame->width - frame->pivotX,
                        YPos - frame->height - frame->pivotY, frame->width,
                        frame->height, frame->sprX, frame->sprY, FLIP_XY,
                        frame->sheetID);
      break;

    default:
      break;
    }
    break;

  case ROTSTYLE_FULL:
    DrawSpriteRotated(entity->direction, XPos, YPos, -frame->pivotX,
                      -frame->pivotY, frame->sprX, frame->sprY, frame->width,
                      frame->height, entity->rotation, frame->sheetID);
    break;

  case ROTSTYLE_45DEG:
    if (entity->rotation >= 0x100)
      DrawSpriteRotated(
          entity->direction, XPos, YPos, -frame->pivotX, -frame->pivotY,
          frame->sprX, frame->sprY, frame->width, frame->height,
          0x200 - ((0x214 - entity->rotation) >> 6 << 6), frame->sheetID);
    else
      DrawSpriteRotated(entity->direction, XPos, YPos, -frame->pivotX,
                        -frame->pivotY, frame->sprX, frame->sprY, frame->width,
                        frame->height, (entity->rotation + 20) >> 6 << 6,
                        frame->sheetID);
    break;

  case ROTSTYLE_STATICFRAMES: {
    if (entity->rotation >= 0x100)
      rotation = 8 - ((532 - entity->rotation) >> 6);
    else
      rotation = (entity->rotation + 20) >> 6;
    int32 frameID = entity->frame;
    switch (rotation) {
    case 0: // 0 deg
    case 8: // 360 deg
      rotation = 0x00;
      break;

    case 1: // 45 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 0;
      else
        rotation = 0x80;
      break;

    case 2: // 90 deg
      rotation = 0x80;
      break;

    case 3: // 135 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 0x80;
      else
        rotation = 0x100;
      break;

    case 4: // 180 deg
      rotation = 0x100;
      break;

    case 5: // 225 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 0x100;
      else
        rotation = 384;
      break;

    case 6: // 270 deg
      rotation = 384;
      break;

    case 7: // 315 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 384;
      else
        rotation = 0;
      break;

    default:
      break;
    }

    frame = &Legacy_animFrames[sprAnim->frameListOffset + frameID];
    DrawSpriteRotated(entity->direction, XPos, YPos, -frame->pivotX,
                      -frame->pivotY, frame->sprX, frame->sprY, frame->width,
                      frame->height, rotation, frame->sheetID);
    break;
  }

  default:
    break;
  }
}

void RSDK::Legacy::v4::DrawFadedFace(void *v, uint32 color, uint32 fogColor,
                                     int32 alpha) { 
  RSDK::Legacy::DrawFace(v, color);
  int32 fog = (fogColor & 0xFFFFFF) | (((1 - alpha) << 23) & 0x7F000000);
  RSDK::Legacy::DrawFace(v, fog);
}
void RSDK::Legacy::v4::DrawTexturedFaceBlended(void *v, uint8 sheetID) {
  Vertex *verts = (Vertex *)v;
  if (verts[0].x < 0 && verts[1].x < 0 && verts[2].x < 0 && verts[3].x < 0)
    return;

  if (verts[0].x > Legacy_GFX_LINESIZE && verts[1].x > Legacy_GFX_LINESIZE &&
      verts[2].x > Legacy_GFX_LINESIZE && verts[3].x > Legacy_GFX_LINESIZE)
    return;

  if (verts[0].y < 0 && verts[1].y < 0 && verts[2].y < 0 && verts[3].y < 0)
    return;

  if (verts[0].y > SCREEN_YSIZE && verts[1].y > SCREEN_YSIZE &&
      verts[2].y > SCREEN_YSIZE && verts[3].y > SCREEN_YSIZE)
    return;

  if (verts[0].x == verts[1].x && verts[1].x == verts[2].x &&
      verts[2].x == verts[3].x)
    return;

  if (verts[0].y == verts[1].y && verts[1].y == verts[2].y &&
      verts[2].y == verts[3].y)
    return;

  int32 vertexA = 0;
  int32 vertexB = 1;
  int32 vertexC = 2;
  int32 vertexD = 3;
  if (verts[1].y < verts[0].y) {
    vertexA = 1;
    vertexB = 0;
  }
  if (verts[2].y < verts[vertexA].y) {
    int32 temp = vertexA;
    vertexA = 2;
    vertexC = temp;
  }
  if (verts[3].y < verts[vertexA].y) {
    int32 temp = vertexA;
    vertexA = 3;
    vertexD = temp;
  }
  if (verts[vertexC].y < verts[vertexB].y) {
    int32 temp = vertexB;
    vertexB = vertexC;
    vertexC = temp;
  }
  if (verts[vertexD].y < verts[vertexB].y) {
    int32 temp = vertexB;
    vertexB = vertexD;
    vertexD = temp;
  }
  if (verts[vertexD].y < verts[vertexC].y) {
    int32 temp = vertexC;
    vertexC = vertexD;
    vertexD = temp;
  }

  int32 faceTop = verts[vertexA].y;
  int32 faceBottom = verts[vertexD].y;
  if (faceTop < 0)
    faceTop = 0;
  if (faceBottom > SCREEN_YSIZE)
    faceBottom = SCREEN_YSIZE;
  for (int32 i = faceTop; i < faceBottom; ++i) {
    RSDK::scanEdgeBuffer[i].start = 100000;
    RSDK::scanEdgeBuffer[i].end = -100000;
  }

  ProcessScanEdgeUV(&verts[vertexA], &verts[vertexB]);
  ProcessScanEdgeUV(&verts[vertexA], &verts[vertexC]);
  ProcessScanEdgeUV(&verts[vertexA], &verts[vertexD]);
  ProcessScanEdgeUV(&verts[vertexB], &verts[vertexC]);
  ProcessScanEdgeUV(&verts[vertexC], &verts[vertexD]);
  ProcessScanEdgeUV(&verts[vertexB], &verts[vertexD]);

  uint16 *frameBuffer =
      &currentScreen->frameBuffer[Legacy_GFX_LINESIZE * faceTop];
  uint8 *pixels = &graphicData[gfxSurface[sheetID].dataPosition];
  int32 shiftwidth = gfxSurface[sheetID].widthShift;
  uint8 *lineBuffer = &gfxLineBuffer[faceTop];
  while (faceTop < faceBottom) {
    Legacy_activePalette = fullPalette[*lineBuffer];
    lineBuffer++;

    int32 startX = RSDK::scanEdgeBuffer[faceTop].start;
    int32 endX = RSDK::scanEdgeBuffer[faceTop].end;
    int32 UPos = scanEdgeBufferU[faceTop].start;
    int32 VPos = scanEdgeBufferV[faceTop].start;

    if (startX >= Legacy_GFX_LINESIZE || endX <= 0) {
      frameBuffer += Legacy_GFX_LINESIZE;
    } else {
      int32 posDifference = endX - startX;
      int32 bufferedUPos = 0;
      int32 bufferedVPos = 0;
      if (endX == startX) {
        bufferedUPos = 0;
        bufferedVPos = 0;
      } else {
        bufferedUPos = (scanEdgeBufferU[faceTop].end - UPos) / posDifference;
        bufferedVPos = (scanEdgeBufferV[faceTop].end - VPos) / posDifference;
      }

      if (endX > Legacy_GFX_LINESIZE_MINUSONE)
        posDifference = Legacy_GFX_LINESIZE_MINUSONE - startX;

      if (startX < 0) {
        posDifference += startX;
        UPos -= startX * bufferedUPos;
        VPos -= startX * bufferedVPos;
        startX = 0;
      }

      uint16 *framebufferPtr = &frameBuffer[startX];
      frameBuffer += Legacy_GFX_LINESIZE;
      int32 counter = posDifference + 1;
      while (counter--) {
        if (UPos < 0)
          UPos = 0;
        if (VPos < 0)
          VPos = 0;
        uint16 index = pixels[(VPos >> 16 << shiftwidth) + (UPos >> 16)];
        if (index > 0)
          *framebufferPtr = ((Legacy_activePalette[index] & 0xF7BC) >> 1) +
                            ((*framebufferPtr & 0xF7BC) >> 1);
        framebufferPtr++;
        UPos += bufferedUPos;
        VPos += bufferedVPos;
      }
    }
    ++faceTop;
  }
}

void RSDK::Legacy::v3::DrawObjectAnimation(void *objScr, void *ent, int32 XPos,
                                           int32 YPos) {
  ObjectScript *objectScript = (ObjectScript *)objScr;
  Entity *entity = (Entity *)ent;
  SpriteAnimation *sprAnim =
      &Legacy_animationList[objectScript->animFile->aniListOffset + entity->animation];
  SpriteFrame *frame = &Legacy_animFrames[sprAnim->frameListOffset + entity->frame];
  int32 rotation = 0;

  switch (sprAnim->rotationStyle) {
  case ROTSTYLE_NONE:
    switch (entity->direction) {
    case FLIP_NONE:
      DrawSpriteFlipped(frame->pivotX + XPos, frame->pivotY + YPos,
                        frame->width, frame->height, frame->sprX, frame->sprY,
                        FLIP_NONE, frame->sheetID);
      break;

    case FLIP_X:
      DrawSpriteFlipped(XPos - frame->width - frame->pivotX,
                        frame->pivotY + YPos, frame->width, frame->height,
                        frame->sprX, frame->sprY, FLIP_X, frame->sheetID);
      break;
    case FLIP_Y:

      DrawSpriteFlipped(frame->pivotX + XPos,
                        YPos - frame->height - frame->pivotY, frame->width,
                        frame->height, frame->sprX, frame->sprY, FLIP_Y,
                        frame->sheetID);
      break;

    case FLIP_XY:
      DrawSpriteFlipped(XPos - frame->width - frame->pivotX,
                        YPos - frame->height - frame->pivotY, frame->width,
                        frame->height, frame->sprX, frame->sprY, FLIP_XY,
                        frame->sheetID);
      break;

    default:
      break;
    }
    break;

  case ROTSTYLE_FULL:
    DrawSpriteRotated(entity->direction, XPos, YPos, -frame->pivotX,
                      -frame->pivotY, frame->sprX, frame->sprY, frame->width,
                      frame->height, entity->rotation, frame->sheetID);
    break;

  case ROTSTYLE_45DEG:
    if (entity->rotation >= 0x100)
      DrawSpriteRotated(
          entity->direction, XPos, YPos, -frame->pivotX, -frame->pivotY,
          frame->sprX, frame->sprY, frame->width, frame->height,
          0x200 - ((0x214 - entity->rotation) >> 6 << 6), frame->sheetID);
    else
      DrawSpriteRotated(entity->direction, XPos, YPos, -frame->pivotX,
                        -frame->pivotY, frame->sprX, frame->sprY, frame->width,
                        frame->height, (entity->rotation + 20) >> 6 << 6,
                        frame->sheetID);
    break;

  case ROTSTYLE_STATICFRAMES: {
    if (entity->rotation >= 0x100)
      rotation = 8 - ((532 - entity->rotation) >> 6);
    else
      rotation = (entity->rotation + 20) >> 6;
    int32 frameID = entity->frame;
    switch (rotation) {
    case 0: // 0 deg
    case 8: // 360 deg
      rotation = 0x00;
      break;

    case 1: // 45 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 0;
      else
        rotation = 0x80;
      break;

    case 2: // 90 deg
      rotation = 0x80;
      break;

    case 3: // 135 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 0x80;
      else
        rotation = 0x100;
      break;

    case 4: // 180 deg
      rotation = 0x100;
      break;

    case 5: // 225 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 0x100;
      else
        rotation = 384;
      break;

    case 6: // 270 deg
      rotation = 384;
      break;

    case 7: // 315 deg
      frameID += sprAnim->frameCount;
      if (entity->direction)
        rotation = 384;
      else
        rotation = 0;
      break;

    default:
      break;
    }

    frame = &Legacy_animFrames[sprAnim->frameListOffset + frameID];
    DrawSpriteRotated(entity->direction, XPos, YPos, -frame->pivotX,
                      -frame->pivotY, frame->sprX, frame->sprY, frame->width,
                      frame->height, rotation, frame->sheetID);
    break;
  }

  default:
    break;
  }
}

void RSDK::Legacy::DrawTextMenuEntry(void *menu, int32 rowID, int32 XPos,
                                     int32 YPos, int32 textHighlight) {
  TextMenu *tMenu = (TextMenu *)menu;
  int32 id = tMenu->entryStart[rowID];
  for (int32 i = 0; i < tMenu->entrySize[rowID]; ++i) {
    DrawSprite(
        XPos + (i << 3) -
            (((tMenu->entrySize[rowID] % 2) & (tMenu->alignment == 2)) * 4),
        YPos, 8, 8, ((tMenu->textData[id] & 0xF) << 3),
        ((tMenu->textData[id] >> 4) << 3) + textHighlight, textMenuSurfaceNo);
    id++;
  }
}
void RSDK::Legacy::DrawStageTextEntry(void *menu, int32 rowID, int32 XPos,
                                      int32 YPos, int32 textHighlight) {
  TextMenu *tMenu = (TextMenu *)menu;
  int32 id = tMenu->entryStart[rowID];
  for (int32 i = 0; i < tMenu->entrySize[rowID]; ++i) {
    if (i == tMenu->entrySize[rowID] - 1) {
      DrawSprite(XPos + (i << 3), YPos, 8, 8,
                 ((tMenu->textData[id] & 0xF) << 3),
                 ((tMenu->textData[id] >> 4) << 3), textMenuSurfaceNo);
    } else {
      DrawSprite(
          XPos + (i << 3), YPos, 8, 8, ((tMenu->textData[id] & 0xF) << 3),
          ((tMenu->textData[id] >> 4) << 3) + textHighlight, textMenuSurfaceNo);
    }
    id++;
  }
}
void RSDK::Legacy::DrawBlendedTextMenuEntry(void *menu, int32 rowID, int32 XPos,
                                            int32 YPos, int32 textHighlight) {
  TextMenu *tMenu = (TextMenu *)menu;
  int32 id = tMenu->entryStart[rowID];
  for (int32 i = 0; i < tMenu->entrySize[rowID]; ++i) {
    DrawBlendedSprite(
        XPos + (i << 3), YPos, 8, 8, ((tMenu->textData[id] & 0xF) << 3),
        ((tMenu->textData[id] >> 4) << 3) + textHighlight, textMenuSurfaceNo);
    id++;
  }
}
void RSDK::Legacy::DrawTextMenu(void *menu, int32 XPos, int32 YPos) {
  TextMenu *tMenu = (TextMenu *)menu;
  int32 cnt = 0;

  if (tMenu->visibleRowCount > 0) {
    cnt = (int32)(tMenu->visibleRowCount + tMenu->visibleRowOffset);
  } else {
    tMenu->visibleRowOffset = 0;
    cnt = (int32)tMenu->rowCount;
  }

  if (tMenu->selectionCount == 3) {
    tMenu->selection2 = -1;
    for (int32 i = 0; i < tMenu->selection1 + 1; ++i) {
      if (tMenu->entryHighlight[i]) {
        tMenu->selection2 = i;
      }
    }
  }

  switch (tMenu->alignment) {
  case 0:
    for (int32 i = (int32)tMenu->visibleRowOffset; i < cnt; ++i) {
      switch (tMenu->selectionCount) {
      case 1:
        if (i == tMenu->selection1)
          DrawTextMenuEntry(tMenu, i, XPos, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, XPos, YPos, 0);
        break;

      case 2:
        if (i == tMenu->selection1 || i == tMenu->selection2)
          DrawTextMenuEntry(tMenu, i, XPos, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, XPos, YPos, 0);
        break;

      case 3:
        if (i == tMenu->selection1)
          DrawTextMenuEntry(tMenu, i, XPos, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, XPos, YPos, 0);

        if (i == tMenu->selection2 && i != tMenu->selection1)
          DrawStageTextEntry(tMenu, i, XPos, YPos, 128);
        break;
      }
      YPos += 8;
    }
    break;

  case 1:
    for (int32 i = (int32)tMenu->visibleRowOffset; i < cnt; ++i) {
      int32 entryX = XPos - (tMenu->entrySize[i] << 3);
      switch (tMenu->selectionCount) {
      case 1:
        if (i == tMenu->selection1)
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 0);
        break;

      case 2:
        if (i == tMenu->selection1 || i == tMenu->selection2)
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 0);
        break;

      case 3:
        if (i == tMenu->selection1)
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 0);

        if (i == tMenu->selection2 && i != tMenu->selection1)
          DrawStageTextEntry(tMenu, i, entryX, YPos, 128);
        break;
      }
      YPos += 8;
    }
    break;

  case 2:
    for (int32 i = (int32)tMenu->visibleRowOffset; i < cnt; ++i) {
      int32 entryX = XPos - (tMenu->entrySize[i] >> 1 << 3);
      switch (tMenu->selectionCount) {
      case 1:
        if (i == tMenu->selection1)
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 0);
        break;
      case 2:
        if (i == tMenu->selection1 || i == tMenu->selection2)
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 0);
        break;
      case 3:
        if (i == tMenu->selection1)
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 128);
        else
          DrawTextMenuEntry(tMenu, i, entryX, YPos, 0);

        if (i == tMenu->selection2 && i != tMenu->selection1)
          DrawStageTextEntry(tMenu, i, entryX, YPos, 128);
        break;
      }
      YPos += 8;
    }
    break;

  default:
    break;
  }
}
