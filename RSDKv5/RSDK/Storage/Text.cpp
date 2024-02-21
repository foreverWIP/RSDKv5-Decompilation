#include "RSDK/Core/RetroEngine.hpp"

using namespace RSDK;

#if RETRO_REV0U
#include "Legacy/TextLegacy.cpp"
#endif

// From here: https://rosettacode.org/wiki/MD5#C

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>

char RSDK::textBuffer[0x400];

uint8 utf8CharSizes[] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                          1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                          1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                          1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                          1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                          1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                          2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6 };

void RSDK::SetString(String *string, const char *text)
{
    if (!*text)
        return;

    int32 newLength = 0;
    for (int32 c = 0; text[c]; ++newLength) c += utf8CharSizes[*text & 0xFF];

    if (!newLength)
        return;

    if (string->size < newLength || !string->chars) {
        string->size = newLength;
        AllocateStorage((void **)&string->chars, sizeof(uint16) * string->size, DATASET_STR, false);
    }

    string->length = newLength;
    for (int32 pos = 0; pos < string->length; ++pos) {
        uint16 c = 0;
        switch (utf8CharSizes[*text & 0xFF]) {
            default: break;

            case 1:
                c = text[0];
                ++text;
                break;

            case 2:
                c = (text[1] & 0x3F) | ((text[0] & 0x1F) << 6);
                text += 2;
                break;

            case 3:
                c = (text[2] & 0x3F) | ((text[1] & 0x3F) << 6) | (text[0] << 12);
                text += 3;
                break;

            case 4:
                c = (text[3] & 0x3F) | ((text[2] & 0x3F) << 6) | (text[1] << 12);
                text += 4;
                break;

            case 5: text += 5; break;

            case 6: text += 6; break;
        }

        string->chars[pos] = c;
    }
}

void RSDK::AppendText(String *string, const char *appendString)
{
    if (!*appendString)
        return;

    int32 len     = 0;
    const char *textBuf = appendString;
    int32 pos;
    for (pos = 0; *textBuf; ++len) pos += utf8CharSizes[*textBuf++ & 0xFF];
    (void)pos;

    if (!len)
        return;

    int32 newSize = len + string->size;
    if (string->size < newSize || !string->chars) {
        if (string->chars) {
            uint16 *charsStore = string->chars;
            AllocateStorage((void **)&string->chars, sizeof(uint16) * newSize, DATASET_STR, false);

            for (int32 c = 0; c < string->length; ++c) string->chars[c] = charsStore[c];
            charsStore = NULL;
        }
        else {
            AllocateStorage((void **)&string->chars, sizeof(uint16) * newSize, DATASET_STR, false);
        }

        string->size = newSize;
    }

    for (int32 c = string->length; c < string->length + len; ++c) {
        uint16 curChar = 0;
        switch (utf8CharSizes[*appendString & 0xFF]) {
            default: break;

            case 1:
                curChar = appendString[0];
                ++appendString;
                break;

            case 2:
                curChar = (appendString[1] & 0x3F) | ((appendString[0] & 0x1F) << 6);
                appendString += 2;
                break;

            case 3:
                curChar = (appendString[2] & 0x3F) | ((appendString[1] & 0x3F) << 6) | (appendString[0] << 12);
                appendString += 3;
                break;

            case 4:
                curChar = (appendString[3] & 0x3F) | ((appendString[2] & 0x3F) << 6) | (appendString[1] << 12);
                appendString += 4;
                break;

            case 5: appendString += 5; break;

            case 6: appendString += 6; break;
        }

        string->chars[c] = curChar;
    }

    string->length = newSize;
}

void RSDK::AppendString(String *string, String *appendString)
{
    uint32 newSize = appendString->length + string->length;

    if (string->size < newSize || !string->chars) {
        if (string->chars) {
            uint16 *charsStore = string->chars;
            AllocateStorage((void **)&string->chars, sizeof(uint16) * newSize, DATASET_STR, false);

            for (int32 c = 0; c < string->length; ++c) string->chars[c] = charsStore[c];
            charsStore = NULL;
        }
        else {
            AllocateStorage((void **)&string->chars, sizeof(uint16) * newSize, DATASET_STR, false);
        }

        string->size = newSize;
    }

    int32 startOffset = string->length;
    string->length += appendString->length;
    for (int32 c = 0, pos = startOffset; pos < string->length; ++pos, ++c) string->chars[pos] = appendString->chars[c];
}

bool32 RSDK::CompareStrings(String *string1, String *string2, bool32 exactMatch)
{
    if (string1->length != string2->length)
        return false;

    if (exactMatch) { // each character has to match
        for (int32 i = 0; i < string1->length; ++i) {
            if (string1->chars[i] != string2->chars[i])
                return false;
        }
    }
    else { // ignore case sensitivity when matching
        if (string1->length <= 0)
            return true;

        for (int32 i = 0; i < string1->length; ++i) {
            if (string1->chars[i] != string2->chars[i]) {
                if (string1->chars[i] != (string2->chars[i] + 0x20) && string1->chars[i] != (string2->chars[i] - 0x20))
                    return false;
            }
        }
    }

    return true;
}

void RSDK::InitStringList(String *stringList, int32 size)
{
    uint16 *text = NULL;

    AllocateStorage((void **)&text, sizeof(uint16) * size, DATASET_STR, false);

    for (int32 c = 0; c < size && c < stringList->length; ++c) text[c] = stringList->chars[c];

    CopyStorage((uint32 **)&stringList->chars, (uint32 **)&text);
    stringList->size = size;
    if (stringList->length > (uint16)size)
        stringList->length = size;
}

void RSDK::LoadStringList(String *stringList, const char *filePath, uint32 charSize)
{
    char fullFilePath[0x40];
    sprintf_s(fullFilePath, sizeof(fullFilePath), "Data/Strings/%s", filePath);

    FileInfo info;
    InitFileInfo(&info);
    if (LoadFile(&info, fullFilePath, FMODE_RB)) {
#if RETRO_REV02
        uint16 header = ReadInt16(&info);
        if (header == 0xFEFF) {
            // UTF-16
            InitStringList(stringList, (info.fileSize >> 1) - 1);
#if !RETRO_USE_ORIGINAL_CODE
            for (int32 c = 0; c < stringList->size; ++c) stringList->chars[c] = ReadInt16(&info);
#else
            // This only works as intended on little-endian CPUs.
            ReadBytes(&info, stringList->chars, stringList->size * sizeof(uint16));
#endif
            stringList->length = stringList->size;
        }
        else {
            // UTF-8
            if (header == 0xEFBB)
                ReadInt8(&info);
            else
                Seek_Set(&info, 0);

            InitStringList(stringList, info.fileSize);

            for (int32 pos = 0; pos < info.fileSize; ++pos) {
                int32 curChar = 0;

                uint8 bit = ReadInt8(&info);
                switch (utf8CharSizes[bit]) {
                    case 1: curChar = bit; break;
                    case 2:
                        curChar = ((bit & 0x1F) << 6);
                        curChar |= (ReadInt8(&info) & 0x3F);
                        break;

                    case 3:
                        curChar = (bit << 12);
                        curChar |= ((ReadInt8(&info) & 0x3F) << 6);
                        curChar |= ReadInt8(&info) & 0x3F;
                        break;

                    case 4:
                        curChar = ReadInt8(&info) << 12;
                        curChar |= ((ReadInt8(&info) & 0x3F) << 6);
                        curChar |= ReadInt8(&info) & 0x3F;
                        break;

                    case 5:
                        pos += 4;
                        Seek_Cur(&info, 4);
                        break;

                    case 6:
                        pos += 5;
                        Seek_Cur(&info, 5);
                        break;

                    default: break;
                }

                stringList->chars[stringList->length++] = curChar;
            }
        }
#else
        switch (charSize) {
            default:
            case 8: // ASCII
                if (stringList->size < info.fileSize) {
                    stringList->size = info.fileSize;
                    AllocateStorage((void **)&stringList->chars, sizeof(uint16) * stringList->size, DATASET_STR, false);
                }
                stringList->length = info.fileSize;
                InitStringList(stringList, info.fileSize);

                for (int32 c = 0; c < stringList->length; ++c) stringList->chars[c] = ReadInt8(&info);
                break;

            case 16: // UTF-16
                if (stringList->size < info.fileSize) {
                    stringList->size = info.fileSize >> 1;
                    AllocateStorage((void **)&stringList->chars, sizeof(uint16) * stringList->size, DATASET_STR, false);
                }
                stringList->length = info.fileSize >> 1;
                InitStringList(stringList, info.fileSize >> 1);

                for (int32 c = 0; c < stringList->length; ++c) stringList->chars[c] = ReadInt16(&info);
                break;
        }
#endif

        CloseFile(&info);
    }
}

bool32 RSDK::SplitStringList(String *splitStrings, String *stringList, int32 startStringID, int32 stringCount)
{
    if (!stringList->size || !stringList->chars)
        return false;

    int32 lastCharPos = 0;
    int32 curStringID = 0;

    bool32 hasSplitString = false;
    for (int32 curCharPos = 0; curCharPos < stringList->length && stringCount > 0; ++curCharPos) {
        if (stringList->chars[curCharPos] == '\n') {
            if (curStringID < startStringID) {
                lastCharPos = curCharPos;
            }
            else {
                uint16 length = curCharPos - lastCharPos;
                if (splitStrings->size < length) {
                    splitStrings->size = length;
                    AllocateStorage((void **)&splitStrings->chars, sizeof(uint16) * length, DATASET_STR, true);
                }
                splitStrings->length = length;

                for (int32 i = 0; i < splitStrings->length; ++i) splitStrings->chars[i] = stringList->chars[lastCharPos++];

                ++splitStrings;
                --stringCount;
                hasSplitString = true;
            }

            ++curStringID;
            ++lastCharPos;
        }
    }

    return hasSplitString;
}

#if RETRO_REV0U
int32 RSDK::FindStringToken(const char *string, const char *token, uint8 stopID)
{
    int32 tokenCharID  = 0;
    bool32 tokenMatch  = true;
    int32 stringCharID = 0;
    int32 foundTokenID = 0;

    while (string[stringCharID]) {
        tokenCharID = 0;
        tokenMatch  = true;
        while (token[tokenCharID]) {
            if (!string[tokenCharID + stringCharID])
                return -1;

            if (string[tokenCharID + stringCharID] != token[tokenCharID])
                tokenMatch = false;

            ++tokenCharID;
        }
        if (tokenMatch && ++foundTokenID == stopID)
            return stringCharID;

        ++stringCharID;
    }
    return -1;
}
#endif