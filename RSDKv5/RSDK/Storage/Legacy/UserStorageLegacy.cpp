

bool32 RSDK::Legacy::ReadSaveRAM() { return SKU::LoadUserFile("SGame.bin", Legacy_saveRAM, sizeof(Legacy_saveRAM)); }
bool32 RSDK::Legacy::WriteSaveRAM() { return SKU::SaveUserFile("SGame.bin", Legacy_saveRAM, sizeof(Legacy_saveRAM)); }

void RSDK::Legacy::v3::SetAchievement(int32 achievementID, int32 achievementDone)
{
    PrintLog(PRINT_NORMAL, "[RSDKv3] Achieved achievement: %d (%d)!", achievementID, achievementDone);
}
void RSDK::Legacy::v3::SetLeaderboard(int32 leaderboardID, int32 score)
{
    PrintLog(PRINT_NORMAL, "[RSDKv3] Setting Leaderboard %d score to %d...", leaderboardID, score);
}

// Native Functions

void RSDK::Legacy::v4::SetAchievement(int32 *achievementID, int32 *status)
{
    if (!achievementID || !status)
        return;

    PrintLog(PRINT_NORMAL, "[RSDKv4] Achieved achievement: %d (%d)!", *achievementID, *status);

    if ((uint32)*achievementID >= achievementList.size())
        return;

    achievementList[*achievementID].achieved = *status ? true : false;
}
void RSDK::Legacy::v4::SetLeaderboard(int32 *leaderboardID, int32 *score)
{
    if (!leaderboardID || !score)
        return;

    PrintLog(PRINT_NORMAL, "[RSDKv4] Setting Leaderboard %d score to %d...", *leaderboardID, *score);
}