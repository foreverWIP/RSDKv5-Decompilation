int32 RSDK::Legacy::vertexCount = 0;
int32 RSDK::Legacy::faceCount   = 0;

RSDK::Matrix RSDK::Legacy::matFinal;
RSDK::Matrix RSDK::Legacy::matWorld;
RSDK::Matrix RSDK::Legacy::matView;
RSDK::Matrix RSDK::Legacy::matTemp;

int32 RSDK::Legacy::projectionX = 136;
int32 RSDK::Legacy::projectionY = 160;

int32 RSDK::Legacy::faceLineStart[SCREEN_YSIZE];
int32 RSDK::Legacy::faceLineEnd[SCREEN_YSIZE];
int32 RSDK::Legacy::faceLineStartU[SCREEN_YSIZE];
int32 RSDK::Legacy::faceLineEndU[SCREEN_YSIZE];
int32 RSDK::Legacy::faceLineStartV[SCREEN_YSIZE];
int32 RSDK::Legacy::faceLineEndV[SCREEN_YSIZE];

RSDK::Legacy::Vertex RSDK::Legacy::vertexBuffer[LEGACY_VERTEXBUFFER_SIZE];
RSDK::Legacy::Vertex RSDK::Legacy::vertexBufferT[LEGACY_VERTEXBUFFER_SIZE];

RSDK::Legacy::Face RSDK::Legacy::faceBuffer[LEGACY_FACEBUFFER_SIZE];

RSDK::Legacy::DrawListEntry3D RSDK::Legacy::drawList3D[LEGACY_FACEBUFFER_SIZE];

int32 RSDK::Legacy::fogColor    = 0;
int32 RSDK::Legacy::fogStrength = 0;

void RSDK::Legacy::ProcessScanEdge(Vertex *vertA, Vertex *vertB)
{
    int32 bottom, top;

    if (vertA->y == vertB->y)
        return;
    if (vertA->y >= vertB->y) {
        top    = vertB->y;
        bottom = vertA->y + 1;
    }
    else {
        top    = vertA->y;
        bottom = vertB->y + 1;
    }
    if (top > SCREEN_YSIZE - 1 || bottom < 0)
        return;
    if (bottom > SCREEN_YSIZE)
        bottom = SCREEN_YSIZE;
    int32 fullX  = vertA->x << 16;
    int32 deltaX = ((vertB->x - vertA->x) << 16) / (vertB->y - vertA->y);
    if (top < 0) {
        fullX -= top * deltaX;
        top = 0;
    }
    for (int32 i = top; i < bottom; ++i) {
        int32 trueX = fullX >> 16;
        if (trueX < faceLineStart[i])
            faceLineStart[i] = trueX;
        if (trueX > faceLineEnd[i])
            faceLineEnd[i] = trueX;
        fullX += deltaX;
    }
}
void RSDK::Legacy::ProcessScanEdgeUV(Vertex *vertA, Vertex *vertB)
{
    int32 bottom, top;

    if (vertA->y == vertB->y)
        return;
    if (vertA->y >= vertB->y) {
        top    = vertB->y;
        bottom = vertA->y + 1;
    }
    else {
        top    = vertA->y;
        bottom = vertB->y + 1;
    }
    if (top > SCREEN_YSIZE - 1 || bottom < 0)
        return;
    if (bottom > SCREEN_YSIZE)
        bottom = SCREEN_YSIZE;

    int32 fullX  = vertA->x << 16;
    int32 fullU  = vertA->u << 16;
    int32 fullV  = vertA->v << 16;
    int32 deltaX = ((vertB->x - vertA->x) << 16) / (vertB->y - vertA->y);

    int32 deltaU = 0;
    if (vertA->u != vertB->u)
        deltaU = ((vertB->u - vertA->u) << 16) / (vertB->y - vertA->y);

    int32 deltaV = 0;
    if (vertA->v != vertB->v) {
        deltaV = ((vertB->v - vertA->v) << 16) / (vertB->y - vertA->y);
    }

    if (top < 0) {
        fullX -= top * deltaX;
        fullU -= top * deltaU;
        fullV -= top * deltaV;
        top = 0;
    }
    for (int32 i = top; i < bottom; ++i) {
        int32 trueX = fullX >> 16;
        if (trueX < faceLineStart[i]) {
            faceLineStart[i]  = trueX;
            faceLineStartU[i] = fullU;
            faceLineStartV[i] = fullV;
        }
        if (trueX > faceLineEnd[i]) {
            faceLineEnd[i]  = trueX;
            faceLineEndU[i] = fullU;
            faceLineEndV[i] = fullV;
        }
        fullX += deltaX;
        fullU += deltaU;
        fullV += deltaV;
    }
}

void RSDK::Legacy::TransformVertexBuffer()
{
    matFinal.values[0][0] = matWorld.values[0][0];
    matFinal.values[0][1] = matWorld.values[0][1];
    matFinal.values[0][2] = matWorld.values[0][2];
    matFinal.values[0][3] = matWorld.values[0][3];

    matFinal.values[1][0] = matWorld.values[1][0];
    matFinal.values[1][1] = matWorld.values[1][1];
    matFinal.values[1][2] = matWorld.values[1][2];
    matFinal.values[1][3] = matWorld.values[1][3];

    matFinal.values[2][0] = matWorld.values[2][0];
    matFinal.values[2][1] = matWorld.values[2][1];
    matFinal.values[2][2] = matWorld.values[2][2];
    matFinal.values[2][3] = matWorld.values[2][3];

    matFinal.values[3][0] = matWorld.values[3][0];
    matFinal.values[3][1] = matWorld.values[3][1];
    matFinal.values[3][2] = matWorld.values[3][2];
    matFinal.values[3][3] = matWorld.values[3][3];
    MatrixMultiply(&matFinal, &matFinal, &matView);

    for (int32 v = 0; v < vertexCount; ++v) {
        int32 vx = vertexBuffer[v].x;
        int32 vy = vertexBuffer[v].y;
        int32 vz = vertexBuffer[v].z;

        vertexBufferT[v].x =
            (vx * matFinal.values[0][0] >> 8) + (vy * matFinal.values[0][1] >> 8) + (vz * matFinal.values[0][2] >> 8) + matFinal.values[0][3];
        vertexBufferT[v].y =
            (vx * matFinal.values[1][0] >> 8) + (vy * matFinal.values[1][1] >> 8) + (vz * matFinal.values[1][2] >> 8) + matFinal.values[1][3];
        vertexBufferT[v].z =
            (vx * matFinal.values[2][0] >> 8) + (vy * matFinal.values[2][1] >> 8) + (vz * matFinal.values[2][2] >> 8) + matFinal.values[2][3];
    }
}

void RSDK::Legacy::TransformVertices(Matrix *matrix, int32 startIndex, int32 endIndex)
{
    for (int32 v = startIndex; v < endIndex; ++v) {
        int32 vx     = vertexBuffer[v].x;
        int32 vy     = vertexBuffer[v].y;
        int32 vz     = vertexBuffer[v].z;
        Vertex *vert = &vertexBuffer[v];
        vert->x      = (vx * matrix->values[0][0] >> 8) + (vy * matrix->values[0][1] >> 8) + (vz * matrix->values[0][2] >> 8) + matrix->values[0][3];
        vert->y      = (vx * matrix->values[1][0] >> 8) + (vy * matrix->values[1][1] >> 8) + (vz * matrix->values[1][2] >> 8) + matrix->values[1][3];
        vert->z      = (vx * matrix->values[2][0] >> 8) + (vy * matrix->values[2][1] >> 8) + (vz * matrix->values[2][2] >> 8) + matrix->values[2][3];
    }
}

void RSDK::Legacy::Sort3DDrawList()
{
    for (int32 i = 0; i < faceCount; ++i) {
        drawList3D[i].depth = (vertexBufferT[faceBuffer[i].d].z + vertexBufferT[faceBuffer[i].c].z + vertexBufferT[faceBuffer[i].b].z
                               + vertexBufferT[faceBuffer[i].a].z)
                              >> 2;
        drawList3D[i].faceID = i;
    }

    for (int32 i = 0; i < faceCount; ++i) {
        for (int32 j = faceCount - 1; j > i; --j) {
            if (drawList3D[j].depth > drawList3D[j - 1].depth) {
                int32 faceID             = drawList3D[j].faceID;
                int32 depth              = drawList3D[j].depth;
                drawList3D[j].faceID     = drawList3D[j - 1].faceID;
                drawList3D[j].depth      = drawList3D[j - 1].depth;
                drawList3D[j - 1].faceID = faceID;
                drawList3D[j - 1].depth  = depth;
            }
        }
    }
}

void RSDK::Legacy::Draw3DScene(int32 spriteSheetID)
{
    Vertex quad[4];
    for (int32 i = 0; i < faceCount; ++i) {
        Face *face = &faceBuffer[drawList3D[i].faceID];
        memset(quad, 0, 4 * sizeof(Vertex));
        switch (face->flag) {
            default: break;
            case FACE_FLAG_TEXTURED_3D:
                if (vertexBufferT[face->a].z > 0 && vertexBufferT[face->b].z > 0 && vertexBufferT[face->c].z > 0 && vertexBufferT[face->d].z > 0) {
                    quad[0].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->a].x / vertexBufferT[face->a].z;
                    quad[0].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->a].y / vertexBufferT[face->a].z;
                    quad[1].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->b].x / vertexBufferT[face->b].z;
                    quad[1].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->b].y / vertexBufferT[face->b].z;
                    quad[2].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->c].x / vertexBufferT[face->c].z;
                    quad[2].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->c].y / vertexBufferT[face->c].z;
                    quad[3].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->d].x / vertexBufferT[face->d].z;
                    quad[3].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->d].y / vertexBufferT[face->d].z;
                    quad[0].u = vertexBuffer[face->a].u;
                    quad[0].v = vertexBuffer[face->a].v;
                    quad[1].u = vertexBuffer[face->b].u;
                    quad[1].v = vertexBuffer[face->b].v;
                    quad[2].u = vertexBuffer[face->c].u;
                    quad[2].v = vertexBuffer[face->c].v;
                    quad[3].u = vertexBuffer[face->d].u;
                    quad[3].v = vertexBuffer[face->d].v;
                    DrawTexturedFace(quad, spriteSheetID);
                }
                break;
            case FACE_FLAG_TEXTURED_2D:
                if (vertexBufferT[face->a].z >= 0 && vertexBufferT[face->b].z >= 0 && vertexBufferT[face->c].z >= 0
                    && vertexBufferT[face->d].z >= 0) {
                    quad[0].x = vertexBufferT[face->a].x;
                    quad[0].y = vertexBufferT[face->a].y;
                    quad[1].x = vertexBufferT[face->b].x;
                    quad[1].y = vertexBufferT[face->b].y;
                    quad[2].x = vertexBufferT[face->c].x;
                    quad[2].y = vertexBufferT[face->c].y;
                    quad[3].x = vertexBufferT[face->d].x;
                    quad[3].y = vertexBufferT[face->d].y;
                    quad[0].u = vertexBuffer[face->a].u;
                    quad[0].v = vertexBuffer[face->a].v;
                    quad[1].u = vertexBuffer[face->b].u;
                    quad[1].v = vertexBuffer[face->b].v;
                    quad[2].u = vertexBuffer[face->c].u;
                    quad[2].v = vertexBuffer[face->c].v;
                    quad[3].u = vertexBuffer[face->d].u;
                    quad[3].v = vertexBuffer[face->d].v;
                    DrawTexturedFace(quad, spriteSheetID);
                }
                break;
            case FACE_FLAG_COLORED_3D:
                if (vertexBufferT[face->a].z > 0 && vertexBufferT[face->b].z > 0 && vertexBufferT[face->c].z > 0 && vertexBufferT[face->d].z > 0) {
                    quad[0].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->a].x / vertexBufferT[face->a].z;
                    quad[0].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->a].y / vertexBufferT[face->a].z;
                    quad[1].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->b].x / vertexBufferT[face->b].z;
                    quad[1].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->b].y / vertexBufferT[face->b].z;
                    quad[2].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->c].x / vertexBufferT[face->c].z;
                    quad[2].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->c].y / vertexBufferT[face->c].z;
                    quad[3].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->d].x / vertexBufferT[face->d].z;
                    quad[3].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->d].y / vertexBufferT[face->d].z;
                    DrawFace(quad, face->color);
                }
                break;
            case FACE_FLAG_COLORED_2D:
                if (vertexBufferT[face->a].z >= 0 && vertexBufferT[face->b].z >= 0 && vertexBufferT[face->c].z >= 0
                    && vertexBufferT[face->d].z >= 0) {
                    quad[0].x = vertexBufferT[face->a].x;
                    quad[0].y = vertexBufferT[face->a].y;
                    quad[1].x = vertexBufferT[face->b].x;
                    quad[1].y = vertexBufferT[face->b].y;
                    quad[2].x = vertexBufferT[face->c].x;
                    quad[2].y = vertexBufferT[face->c].y;
                    quad[3].x = vertexBufferT[face->d].x;
                    quad[3].y = vertexBufferT[face->d].y;
                    DrawFace(quad, face->color);
                }
                break;
            case FACE_FLAG_FADED:
                if (vertexBufferT[face->a].z > 0 && vertexBufferT[face->b].z > 0 && vertexBufferT[face->c].z > 0 && vertexBufferT[face->d].z > 0) {
                    quad[0].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->a].x / vertexBufferT[face->a].z;
                    quad[0].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->a].y / vertexBufferT[face->a].z;
                    quad[1].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->b].x / vertexBufferT[face->b].z;
                    quad[1].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->b].y / vertexBufferT[face->b].z;
                    quad[2].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->c].x / vertexBufferT[face->c].z;
                    quad[2].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->c].y / vertexBufferT[face->c].z;
                    quad[3].x = SCREEN_CENTERX + projectionX * vertexBufferT[face->d].x / vertexBufferT[face->d].z;
                    quad[3].y = SCREEN_CENTERY - projectionY * vertexBufferT[face->d].y / vertexBufferT[face->d].z;

                    int32 fogStr = 0;
                    if ((drawList3D[i].depth - 0x8000) >> 8 >= 0)
                        fogStr = (drawList3D[i].depth - 0x8000) >> 8;
                    if (fogStr > fogStrength)
                        fogStr = fogStrength;

                    RSDK::Legacy::v4::DrawFadedFace(quad, face->color, fogColor, 0xFF - fogStr);
                }
                break;
            case FACE_FLAG_TEXTURED_C:
                if (vertexBufferT[face->a].z > 0) {
                    // [face->a].uv == sprite center
                    // [face->b].uv == ???
                    // [face->c].uv == sprite extend (how far to each edge X & Y)
                    // [face->d].uv == unused

                    quad[0].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x - vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[0].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y + vertexBuffer[face->b].v) / vertexBufferT[face->a].z;
                    quad[1].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x + vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[1].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y + vertexBuffer[face->b].v) / vertexBufferT[face->a].z;
                    quad[2].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x - vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[2].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y - vertexBuffer[face->b].v) / vertexBufferT[face->a].z;
                    quad[3].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x + vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[3].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y - vertexBuffer[face->b].v) / vertexBufferT[face->a].z;

                    quad[0].u = vertexBuffer[face->a].u - vertexBuffer[face->c].u;
                    quad[0].v = vertexBuffer[face->a].v - vertexBuffer[face->c].v;
                    quad[1].u = vertexBuffer[face->a].u + vertexBuffer[face->c].u;
                    quad[1].v = vertexBuffer[face->a].v - vertexBuffer[face->c].v;
                    quad[2].u = vertexBuffer[face->a].u - vertexBuffer[face->c].u;
                    quad[2].v = vertexBuffer[face->a].v + vertexBuffer[face->c].v;
                    quad[3].u = vertexBuffer[face->a].u + vertexBuffer[face->c].u;
                    quad[3].v = vertexBuffer[face->a].v + vertexBuffer[face->c].v;

                    DrawTexturedFace(quad, spriteSheetID);
                }
                break;
            case FACE_FLAG_TEXTURED_C_BLEND:
                if (vertexBufferT[face->a].z > 0) {
                    // See above, its the same just blended

                    quad[0].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x - vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[0].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y + vertexBuffer[face->b].v) / vertexBufferT[face->a].z;
                    quad[1].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x + vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[1].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y + vertexBuffer[face->b].v) / vertexBufferT[face->a].z;
                    quad[2].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x - vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[2].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y - vertexBuffer[face->b].v) / vertexBufferT[face->a].z;
                    quad[3].x = SCREEN_CENTERX + projectionX * (vertexBufferT[face->a].x + vertexBuffer[face->b].u) / vertexBufferT[face->a].z;
                    quad[3].y = SCREEN_CENTERY - projectionY * (vertexBufferT[face->a].y - vertexBuffer[face->b].v) / vertexBufferT[face->a].z;

                    quad[0].u = vertexBuffer[face->a].u - vertexBuffer[face->c].u;
                    quad[0].v = vertexBuffer[face->a].v - vertexBuffer[face->c].v;
                    quad[1].u = vertexBuffer[face->a].u + vertexBuffer[face->c].u;
                    quad[1].v = vertexBuffer[face->a].v - vertexBuffer[face->c].v;
                    quad[2].u = vertexBuffer[face->a].u - vertexBuffer[face->c].u;
                    quad[2].v = vertexBuffer[face->a].v + vertexBuffer[face->c].v;
                    quad[3].u = vertexBuffer[face->a].u + vertexBuffer[face->c].u;
                    quad[3].v = vertexBuffer[face->a].v + vertexBuffer[face->c].v;

                    RSDK::Legacy::v4::DrawTexturedFaceBlended(quad, spriteSheetID);
                }
                break;
            case FACE_FLAG_3DSPRITE:
                if (vertexBufferT[face->a].z > 0) {
                    int32 xpos = SCREEN_CENTERX + projectionX * vertexBufferT[face->a].x / vertexBufferT[face->a].z;
                    int32 ypos = SCREEN_CENTERY - projectionY * vertexBufferT[face->a].y / vertexBufferT[face->a].z;

                    RSDK::Legacy::v4::ObjectScript *scriptInfo = &RSDK::Legacy::v4::objectScriptList[vertexBuffer[face->a].u];
                    SpriteFrame *frame       = &Legacy_scriptFrames[scriptInfo->frameListOffset + vertexBuffer[face->b].u];

                    switch (vertexBuffer[face->a].v) {
                        case FX_SCALE:
                            DrawSpriteScaled(vertexBuffer[face->b].v, xpos, ypos, -frame->pivotX, -frame->pivotY, vertexBuffer[face->c].u,
                                             vertexBuffer[face->c].u, frame->width, frame->height, frame->sprX, frame->sprY,
                                             scriptInfo->spriteSheetID);
                            break;
                        case FX_ROTATE:
                            DrawSpriteRotated(vertexBuffer[face->b].v, xpos, ypos, -frame->pivotX, -frame->pivotY, frame->sprX, frame->sprY,
                                              frame->width, frame->height, vertexBuffer[face->c].v, scriptInfo->spriteSheetID);
                            break;
                        case FX_ROTOZOOM:
                            DrawSpriteRotozoom(vertexBuffer[face->b].v, xpos, ypos, -frame->pivotX, -frame->pivotY, frame->sprX, frame->sprY,
                                               frame->width, frame->height, vertexBuffer[face->c].v, vertexBuffer[face->c].u,
                                               scriptInfo->spriteSheetID);
                            break;
                    }
                }
                break;
        }
    }
}