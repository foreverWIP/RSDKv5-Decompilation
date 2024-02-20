namespace RSDK {
#include "v3/RetroEnginev3.hpp"
#include "v4/RetroEnginev4.hpp"

namespace Legacy {
enum RetroStates {
    ENGINE_DEVMENU     = 0,
    ENGINE_MAINGAME    = 1,
    ENGINE_INITDEVMENU = 2,

    ENGINE_SCRIPTERROR = 4,
};

enum DeviceTypes { DEVICE_STANDARD = 0, DEVICE_MOBILE = 1 };

enum GamePlatformID {
    LEGACY_RETRO_WIN      = 0,
    LEGACY_RETRO_OSX      = 1,
    LEGACY_RETRO_XBOX_360 = 2,
    LEGACY_RETRO_PS3      = 3,
    LEGACY_RETRO_iOS      = 4,
    LEGACY_RETRO_ANDROID  = 5,
    LEGACY_RETRO_WP7      = 6
};

enum RetroLanguages {
    LEGACY_LANGUAGE_EN = 0,
    LEGACY_LANGUAGE_FR = 1,
    LEGACY_LANGUAGE_IT = 2,
    LEGACY_LANGUAGE_DE = 3,
    LEGACY_LANGUAGE_ES = 4,
    LEGACY_LANGUAGE_JP = 5,
    LEGACY_LANGUAGE_PT = 6,
    LEGACY_LANGUAGE_RU = 7,
    LEGACY_LANGUAGE_KO = 8,
    LEGACY_LANGUAGE_ZH = 9,
    LEGACY_LANGUAGE_ZS = 10
};
}
}

#define LEGACY_RETRO_USE_HAPTICS (1)

extern "C" {
    extern int32 legacy_gameMode;
    extern bool32 legacy_usingBytecode;

    extern bool32 legacy_trialMode;
    extern int32 legacy_gamePlatformID;
    extern int32 legacy_deviceType;
    extern bool32 legacy_onlineActive;
    extern int32 legacy_language;
#if LEGACY_RETRO_USE_HAPTICS
    extern bool32 legacy_hapticsEnabled;
#endif

    extern int32 sinM7LookupTable[0x200];
    extern int32 cosM7LookupTable[0x200];
}

void CalculateTrigAnglesM7();