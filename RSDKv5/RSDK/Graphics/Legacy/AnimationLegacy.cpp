

void RSDK::Legacy::LoadAnimationFile(char *filePath)
{
    FileInfo info;
    InitFileInfo(&info);

    if (LoadFile(&info, filePath, FMODE_RB)) {
        uint8 sheetIDs[24];
        sheetIDs[0] = 0;

        uint8 sheetCount = ReadInt8(&info);

        for (int32 s = 0; s < sheetCount; ++s) {
            char sheetPath[0x21];
            ReadString(&info, sheetPath);
            sheetIDs[s] = AddGraphicsFile(sheetPath);
        }

        AnimationFile *animFile = &Legacy_animationFileList[Legacy_animationFileCount];
        animFile->animCount     = ReadInt8(&info);
        animFile->aniListOffset = Legacy_animationCount;

        for (int32 a = 0; a < animFile->animCount; ++a) {
            SpriteAnimation *anim = &Legacy_animationList[Legacy_animationCount++];
            anim->frameListOffset = Legacy_animFrameCount;

            ReadString(&info, anim->name);
            anim->frameCount    = ReadInt8(&info);
            anim->speed         = ReadInt8(&info);
            anim->loopPoint     = ReadInt8(&info);
            anim->rotationStyle = ReadInt8(&info);

            for (int32 f = 0; f < anim->frameCount; ++f) {
                SpriteFrame *frame = &Legacy_animFrames[Legacy_animFrameCount++];
                frame->sheetID     = sheetIDs[ReadInt8(&info)];
                frame->hitboxID    = ReadInt8(&info);
                frame->sprX        = ReadInt8(&info);
                frame->sprY        = ReadInt8(&info);
                frame->width       = ReadInt8(&info);
                frame->height      = ReadInt8(&info);
                frame->pivotX      = (int8)ReadInt8(&info);
                frame->pivotY      = (int8)ReadInt8(&info);
            }

            // 90 Degree (Extra rotation Frames) rotation
            if (anim->rotationStyle == ROTSTYLE_STATICFRAMES)
                anim->frameCount >>= 1;
        }

        animFile->hitboxListOffset = Legacy_hitboxCount;
        int32 hbCount              = ReadInt8(&info);
        for (int32 h = 0; h < hbCount; ++h) {
            Hitbox *hitbox = &Legacy_hitboxList[Legacy_hitboxCount++];
            for (int32 d = 0; d < LEGACY_HITBOX_DIR_COUNT; ++d) {
                hitbox->left[d]   = ReadInt8(&info);
                hitbox->top[d]    = ReadInt8(&info);
                hitbox->right[d]  = ReadInt8(&info);
                hitbox->bottom[d] = ReadInt8(&info);
            }
        }

        CloseFile(&info);
    }
}
void RSDK::Legacy::ClearAnimationData()
{
    for (int32 f = 0; f < LEGACY_SPRITEFRAME_COUNT; ++f) MEM_ZERO(Legacy_scriptFrames[f]);
    for (int32 f = 0; f < LEGACY_SPRITEFRAME_COUNT; ++f) MEM_ZERO(Legacy_animFrames[f]);
    for (int32 h = 0; h < LEGACY_HITBOX_COUNT; ++h) MEM_ZERO(Legacy_hitboxList[h]);
    for (int32 a = 0; a < LEGACY_ANIMATION_COUNT; ++a) MEM_ZERO(Legacy_animationList[a]);
    for (int32 a = 0; a < LEGACY_ANIFILE_COUNT; ++a) MEM_ZERO(Legacy_animationFileList[a]);

    Legacy_scriptFrameCount   = 0;
    Legacy_animFrameCount     = 0;
    Legacy_animationCount     = 0;
    Legacy_animationFileCount = 0;
    Legacy_hitboxCount        = 0;
}

RSDK::Legacy::AnimationFile *RSDK::Legacy::AddAnimationFile(char *filePath)
{
    char path[0x80];
    StrCopy(path, "Data/Animations/");
    StrAdd(path, filePath);

    for (int32 a = 0; a < LEGACY_ANIFILE_COUNT; ++a) {
        if (StrLength(Legacy_animationFileList[a].fileName) <= 0) {
            StrCopy(Legacy_animationFileList[a].fileName, filePath);
            LoadAnimationFile(path);
            ++Legacy_animationFileCount;
            return &Legacy_animationFileList[a];
        }

        if (StrComp(Legacy_animationFileList[a].fileName, filePath))
            return &Legacy_animationFileList[a];
    }

    return NULL;
}

void RSDK::Legacy::v3::ProcessObjectAnimation(void *objScr, void *ent)
{
    Legacy::v3::ObjectScript *objectScript = (Legacy::v3::ObjectScript *)objScr;
    Legacy::v3::Entity *entity             = (Legacy::v3::Entity *)ent;
    Legacy::SpriteAnimation *sprAnim       = &Legacy_animationList[objectScript->animFile->aniListOffset + entity->animation];

    if (entity->animationSpeed <= 0) {
        entity->animationTimer += sprAnim->speed;
    }
    else {
        if (entity->animationSpeed > 0xF0)
            entity->animationSpeed = 0xF0;
        entity->animationTimer += entity->animationSpeed;
    }

    if (entity->animation != entity->prevAnimation) {
        entity->prevAnimation  = entity->animation;
        entity->frame          = 0;
        entity->animationTimer = 0;
        entity->animationSpeed = 0;
    }

    if (entity->animationTimer >= 0xF0) {
        entity->animationTimer -= 0xF0;
        ++entity->frame;
    }

    if (entity->frame >= sprAnim->frameCount)
        entity->frame = sprAnim->loopPoint;
}

void RSDK::Legacy::v4::ProcessObjectAnimation(void *objScr, void *ent)
{
    Legacy::v4::ObjectScript *objectScript = (Legacy::v4::ObjectScript *)objScr;
    Legacy::v4::Entity *entity             = (Legacy::v4::Entity *)ent;
    Legacy::SpriteAnimation *sprAnim       = &Legacy_animationList[objectScript->animFile->aniListOffset + entity->animation];

    if (entity->animationSpeed <= 0) {
        entity->animationTimer += sprAnim->speed;
    }
    else {
        if (entity->animationSpeed > 0xF0)
            entity->animationSpeed = 0xF0;
        entity->animationTimer += entity->animationSpeed;
    }

    if (entity->animation != entity->prevAnimation) {
        entity->prevAnimation  = entity->animation;
        entity->frame          = 0;
        entity->animationTimer = 0;
        entity->animationSpeed = 0;
    }

    if (entity->animationTimer >= 0xF0) {
        entity->animationTimer -= 0xF0;
        ++entity->frame;
    }

    if (entity->frame >= sprAnim->frameCount)
        entity->frame = sprAnim->loopPoint;
}