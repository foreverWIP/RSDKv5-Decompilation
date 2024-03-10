
namespace Legacy
{

#define LEGACY_GLOBALVAR_COUNT (0x100)

#define LEGACY_SAVEDATA_SIZE (0x2000)

#if RETRO_USE_MOD_LOADER
#define LEGACY_v4_NATIIVEFUNCTION_COUNT (0x30)
#else
#define LEGACY_v4_NATIIVEFUNCTION_COUNT (0x10)
#endif

struct GlobalVariable {
    char name[0x20];
    int32 value;
};

extern "C" {
    extern void *Legacy_nativeFunction[LEGACY_v4_NATIIVEFUNCTION_COUNT];
    extern int32 Legacy_nativeFunctionCount;

    extern int32 Legacy_globalVariablesCount;
    extern GlobalVariable Legacy_globalVariables[LEGACY_GLOBALVAR_COUNT];

    extern int32 Legacy_saveRAM[LEGACY_SAVEDATA_SIZE];

    int32 Legacy_GetGlobalVariableByName(const char *name);
    void Legacy_SetGlobalVariableByName(const char *name, int32 value);
    int32 Legacy_GetGlobalVariableID(const char *name);
}

#define AddNativeFunction(name, funcPtr)                                                                                                             \
    if (Legacy_nativeFunctionCount < LEGACY_v4_NATIIVEFUNCTION_COUNT) {                                                                                     \
        Legacy_SetGlobalVariableByName(name, Legacy_nativeFunctionCount);                                                                                          \
        Legacy_nativeFunction[Legacy_nativeFunctionCount++] = (void *)funcPtr;                                                                                     \
    }

bool32 ReadSaveRAM();
bool32 WriteSaveRAM();

namespace v3
{

void SetAchievement(int32 achievementID, int32 achievementDone);
void SetLeaderboard(int32 leaderboardID, int32 score);
inline void LoadAchievementsMenu() {}
inline void LoadLeaderboardsMenu() {}

} // namespace v3

namespace v4
{
// Native Functions
void SetAchievement(int32 *achievementID, int32 *status);
void SetLeaderboard(int32 *leaderboardID, int32 *score);

extern "C" {
    void HapticEffect(int32 *id, int32 *unknown1, int32 *unknown2, int32 *unknown3);
    void NotifyCallback(int32 *callback, int32 *param1, int32 *param2, int32 *param3);
}

} // namespace v4

} // namespace Legacy