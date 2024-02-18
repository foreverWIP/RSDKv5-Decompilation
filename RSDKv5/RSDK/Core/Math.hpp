#ifndef MATH_H
#define MATH_H

extern "C" {
    extern int32 sin1024LookupTable[0x400];
    extern int32 cos1024LookupTable[0x400];
    extern int32 tan1024LookupTable[0x400];
    extern int32 asin1024LookupTable[0x400];
    extern int32 acos1024LookupTable[0x400];

    extern int32 sin512LookupTable[0x200];
    extern int32 cos512LookupTable[0x200];
    extern int32 tan512LookupTable[0x200];
    extern int32 asin512LookupTable[0x200];
    extern int32 acos512LookupTable[0x200];

    extern int32 sin256LookupTable[0x100];
    extern int32 cos256LookupTable[0x100];
    extern int32 tan256LookupTable[0x100];
    extern int32 asin256LookupTable[0x100];
    extern int32 acos256LookupTable[0x100];

    extern uint8 arcTan256LookupTable[0x100 * 0x100];

    void RSDK_CalculateTrigAngles(void);

    int32 RSDK_Sin1024(int32 angle);
    int32 RSDK_Cos1024(int32 angle);
    int32 RSDK_Tan1024(int32 angle);
    int32 RSDK_ASin1024(int32 angle);
    int32 RSDK_ACos1024(int32 angle);

    int32 RSDK_Sin256(int32 angle);
    int32 RSDK_Cos256(int32 angle);
    int32 RSDK_Tan256(int32 angle);
    int32 RSDK_ASin256(int32 angle);
    int32 RSDK_ACos256(int32 angle);

    int32 RSDK_Sin512(int32 angle);
    int32 RSDK_Cos512(int32 angle);
    int32 RSDK_Tan512(int32 angle);
    int32 RSDK_ASin512(int32 angle);
    int32 RSDK_ACos512(int32 angle);

    // Get Arc Tan value
    uint8 RSDK_ArcTanLookup(int32 x, int32 y);

    extern uint32 randSeed;

    void RSDK_SetRandSeed(int32 key);
    int32 RSDK_Rand(int32 min, int32 max);
    int32 RSDK_RandSeeded(int32 min, int32 max, int32 *randSeed);
}

namespace RSDK
{

// not "math" but works best here
#define INT_TO_VOID(x) (void *)(size_t)(x)
#define VOID_TO_INT(x) (int32)(size_t)(x)

#define MIN(a, b)                      ((a) < (b) ? (a) : (b))
#define MAX(a, b)                      ((a) > (b) ? (a) : (b))
#define CLAMP(value, minimum, maximum) (((value) < (minimum)) ? (minimum) : (((value) > (maximum)) ? (maximum) : (value)))

#define TO_FIXED(x)   ((x) << 16)
#define FROM_FIXED(x) ((x) >> 16)

struct Vector2 {
    int32 x;
    int32 y;
};

#define MEM_ZERO(x) memset(&(x), 0, sizeof((x)))

// Setup angles
void CalculateTrigAngles();

inline int32 Sin1024(int32 angle) { return RSDK_Sin1024(angle); }
inline int32 Cos1024(int32 angle) { return RSDK_Cos1024(angle); }
inline int32 Tan1024(int32 angle) { return RSDK_Tan1024(angle); }
inline int32 ASin1024(int32 angle) { return RSDK_ASin1024(angle); }
inline int32 ACos1024(int32 angle) { return RSDK_ACos1024(angle); }

inline int32 Sin512(int32 angle) { return RSDK_Sin512(angle); }
inline int32 Cos512(int32 angle) { return RSDK_Cos512(angle); }
inline int32 Tan512(int32 angle) { return RSDK_Tan512(angle); }
inline int32 ASin512(int32 angle) { return RSDK_ASin512(angle); }
inline int32 ACos512(int32 angle) { return RSDK_ACos512(angle); }

inline int32 Sin256(int32 angle) { return RSDK_Sin256(angle); }
inline int32 Cos256(int32 angle) { return RSDK_Cos256(angle); }
inline int32 Tan256(int32 angle) { return RSDK_Tan256(angle); }
inline int32 ASin256(int32 angle) { return RSDK_ASin256(angle); }
inline int32 ACos256(int32 angle) { return RSDK_ACos256(angle); }

uint8 ArcTanLookup(int32 x, int32 y);

inline void SetRandSeed(int32 key) { return RSDK_SetRandSeed(key); }
inline int32 Rand(int32 min, int32 max) { return RSDK_Rand(min, max); }
inline int32 RandSeeded(int32 min, int32 max, int32 *randSeed) { return RSDK_RandSeeded(min, max, randSeed); }

} // namespace RSDK

#endif // !MATH_H
