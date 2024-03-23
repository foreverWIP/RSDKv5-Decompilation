#ifndef TEXT_H
#define TEXT_H

extern "C" {
    void GenerateHashMD5(uint32 *buffer, char *textBuffer, int32 textBufferLen);
    void GenerateHashCRC(uint32 *id, char *inputString);
}

namespace RSDK
{

struct GameVersionInfo {
    char gameTitle[0x40];
    char gameSubtitle[0x100];
    char version[0x10];
#if !RETRO_REV02
    uint8 platform;
    uint8 language;
    uint8 region;
#endif
};

extern GameVersionInfo gameVerInfo;

struct String {
    uint16 *chars; // text
    uint16 length;  // string length
    uint16 size;    // total alloc length
};

#if RETRO_REV0U
extern "C" {
    void StrCopy(char *dest, const char *src);
    void StrAdd(char *dest, const char *src);
    bool StrComp(const char *stringA, const char *stringB);
    int32 StrLength(const char *string);
    int32 FindStringToken(const char *string, const char *token, uint8 stopID);
}
#endif

extern "C" {
    extern char textBuffer[0x400];
}

#define RETRO_HASH_MD5(name) uint32 name[4]
#define HASH_SIZE_MD5        (4 * sizeof(uint32))
#define HASH_MATCH_MD5(a, b) (memcmp(a, b, HASH_SIZE_MD5) == 0)
// this is NOT thread-safe!
#define GEN_HASH_MD5(text, hash)                                                                                                                     \
    strcpy(textBuffer, text);                                                                                                                        \
    GenerateHashMD5(hash, textBuffer, (int32)strlen(textBuffer))
// this one is but assumes buffer has already been setup
#define GEN_HASH_MD5_BUFFER(buffer, hash) GenerateHashMD5(hash, buffer, (int32)strlen(buffer))
#define HASH_COPY_MD5(dst, src) memcpy(dst, src, HASH_SIZE_MD5)
#define HASH_CLEAR_MD5(hash)    MEM_ZERO(hash)

extern "C" {
    void InitString(String *string, const char *text, uint32 textLength);
    void SetString(String *string, const char *text);
    void CopyString(String *dst, String *src);
    void GetCString(char *destChars, String *string);
}

void AppendText(String *string, const char *appendText);
void AppendString(String *string, String *appendString);
extern "C" {
    bool32 CompareStrings(String *string1, String *string2, bool32 exactMatch);
}

void InitStringList(String *stringList, int32 size);
void LoadStringList(String *stringList, const char *filePath, uint32 charSize);
bool32 SplitStringList(String *splitStrings, String *stringList, int32 startStringID, int32 stringCount);

#if RETRO_REV0U
#include "Legacy/TextLegacy.hpp"
#endif

} // namespace RSDK

#endif
