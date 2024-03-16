
namespace Legacy
{

#define LEGACY_ANIFILE_COUNT     (0x100)
#define LEGACY_ANIMATION_COUNT   (0x400)
#define LEGACY_SPRITEFRAME_COUNT (0x1000)

#define LEGACY_HITBOX_COUNT     (0x20)
#define LEGACY_HITBOX_DIR_COUNT (0x8)

enum AnimRotationFlags { ROTSTYLE_NONE, ROTSTYLE_FULL, ROTSTYLE_45DEG, ROTSTYLE_STATICFRAMES };

struct AnimationFile {
    char fileName[0x20];
    int32 animCount;
    int32 aniListOffset;
    int32 hitboxListOffset;
};

struct SpriteAnimation {
    char name[16];
    uint8 frameCount;
    uint8 speed;
    uint8 loopPoint;
    uint8 rotationStyle;
    int32 frameListOffset;
};

struct SpriteFrame {
    int32 sprX;
    int32 sprY;
    int32 width;
    int32 height;
    int32 pivotX;
    int32 pivotY;
    uint8 sheetID;
    uint8 hitboxID;
};

struct Hitbox {
    int8 left[LEGACY_HITBOX_DIR_COUNT];
    int8 top[LEGACY_HITBOX_DIR_COUNT];
    int8 right[LEGACY_HITBOX_DIR_COUNT];
    int8 bottom[LEGACY_HITBOX_DIR_COUNT];
};

extern "C" {
    extern AnimationFile Legacy_animationFileList[LEGACY_ANIFILE_COUNT];
    extern int32 Legacy_animationFileCount;

    extern SpriteFrame Legacy_scriptFrames[LEGACY_SPRITEFRAME_COUNT];
    extern int32 Legacy_scriptFrameCount;

    extern SpriteFrame Legacy_animFrames[LEGACY_SPRITEFRAME_COUNT];
    extern int32 Legacy_animFrameCount;
    extern SpriteAnimation Legacy_animationList[LEGACY_ANIMATION_COUNT];
    extern int32 Legacy_animationCount;
    extern Hitbox Legacy_hitboxList[LEGACY_HITBOX_COUNT];
    extern int32 Legacy_hitboxCount;
}

void LoadAnimationFile(char *filePath);
void ClearAnimationData();

AnimationFile *AddAnimationFile(char *filePath);

inline AnimationFile *GetDefaultAnimationRef() { return &Legacy_animationFileList[0]; }

namespace v3
{
void ProcessObjectAnimation(void *objScr, void *ent);
}

namespace v4
{
void ProcessObjectAnimation(void *objScr, void *ent);
}

} // namespace Legacy