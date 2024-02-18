#include "RSDK/Core/RetroEngine.hpp"
#include <math.h>

using namespace RSDK;

void RSDK::CalculateTrigAngles()
{
    RSDK_CalculateTrigAngles();
}

uint8 RSDK::ArcTanLookup(int32 X, int32 Y)
{
    return RSDK_ArcTanLookup(X, Y);
}
