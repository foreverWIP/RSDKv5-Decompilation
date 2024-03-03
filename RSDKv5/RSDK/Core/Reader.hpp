#ifndef READER_H
#define READER_H

#if RETRO_RENDERDEVICE_SDL2 || RETRO_AUDIODEVICE_SDL2 || RETRO_INPUTDEVICE_SDL2
#define FileIO                                          SDL_RWops
#define fOpen(path, mode)                               SDL_RWFromFile(path, mode)
#define fRead(buffer, elementSize, elementCount, file)  SDL_RWread(file, buffer, elementSize, elementCount)
#define fSeek(file, offset, whence)                     SDL_RWseek(file, offset, whence)
#define fTell(file)                                     SDL_RWtell(file)
#define fClose(file)                                    SDL_RWclose(file)
#define fWrite(buffer, elementSize, elementCount, file) SDL_RWwrite(file, buffer, elementSize, elementCount)
#else
#define FileIO                                          FILE
#define fOpen(path, mode)                               fopen(path, mode)
#define fRead(buffer, elementSize, elementCount, file)  fread(buffer, elementSize, elementCount, file)
#define fSeek(file, offset, whence)                     fseek(file, offset, whence)
#define fTell(file)                                     ftell(file)
#define fClose(file)                                    fclose(file)
#define fWrite(buffer, elementSize, elementCount, file) fwrite(buffer, elementSize, elementCount, file)
#endif

#if RETRO_PLATFORM == RETRO_ANDROID
#undef fOpen
FileIO *fOpen(const char *path, const char *mode);
#endif

#include <miniz/miniz.h>

#define RSDK_SIGNATURE_RSDK (0x4B445352) // "RSDK"
#if RETRO_REV0U
#define RSDK_SIGNATURE_DATA (0x61746144) // "Data"
#endif

#define DATAFILE_COUNT (0x1000)
#define DATAPACK_COUNT (4)

enum Scopes {
    SCOPE_NONE,
    SCOPE_GLOBAL,
    SCOPE_STAGE,
};

struct FileInfo {
    int32 fileSize;
    int32 externalFile;
    FileIO *file;
    uint8 *fileBuffer;
    int32 readPos;
    int32 fileOffset;
    uint8 usingFileBuffer;
    uint8 encrypted;
    uint8 eNybbleSwap;
    uint8 encryptionKeyA[0x10];
    uint8 encryptionKeyB[0x10];
    uint8 eKeyPosA;
    uint8 eKeyPosB;
    uint8 eKeyNo;
};

struct RSDKFileInfo {
    RETRO_HASH_MD5(hash);
    int32 size;
    int32 offset;
    uint8 encrypted;
    uint8 useFileBuffer;
    int32 packID;
};

struct RSDKContainer {
    char name[0x100];
    uint8 *fileBuffer;
    int32 fileCount;
};

namespace RSDK
{

extern RSDKFileInfo dataFileList[DATAFILE_COUNT];
extern RSDKContainer dataPacks[DATAPACK_COUNT];

extern uint8 dataPackCount;
extern uint16 dataFileListCount;

extern char gameLogicName[0x200];

extern bool32 useDataPack;

#if RETRO_REV0U
void DetectEngineVersion();
#endif
bool32 LoadDataPack(const char *filename, size_t fileOffset, bool32 useBuffer);
bool32 OpenDataFile(FileInfo *info, const char *filename);

enum FileModes { FMODE_NONE, FMODE_RB, FMODE_WB, FMODE_RB_PLUS };

static const char *openModes[3] = { "rb", "wb", "rb+" };

bool32 LoadFile(FileInfo *info, const char *filename, uint8 fileMode);


extern "C" {
    void GenerateELoadKeys(FileInfo *info, const char *key1, int32 key2);
    void InitFileInfo(FileInfo *info);
    void CloseFile(FileInfo *info);
    void Seek_Set(FileInfo *info, int32 count);
    void Seek_Cur(FileInfo *info, int32 count);
    size_t ReadBytes(FileInfo *info, void *data, int32 count);
    uint8 ReadInt8(FileInfo *info);
    int16 ReadInt16(FileInfo *info);
    int32 ReadInt32(FileInfo *info, bool32 swapEndian);
    int64 ReadInt64(FileInfo *info);
    float ReadSingle(FileInfo *info);
    void ReadString(FileInfo *info, char *buffer);
}

inline int32 Uncompress(uint8 **cBuffer, int32 cSize, uint8 **buffer, int32 size)
{
    if (!buffer || !cBuffer)
        return 0;

    uLongf cLen    = cSize;
    uLongf destLen = size;

    int32 result = uncompress(*buffer, &destLen, *cBuffer, cLen);
    (void)result;

    return (int32)destLen;
}

// The buffer passed in parameter is allocated here, so it's up to the caller to free it once it goes unused
inline int32 ReadCompressed(FileInfo *info, uint8 **buffer)
{
    if (!buffer)
        return 0;

    uint32 cSize  = ReadInt32(info, false) - 4;
    uint32 sizeBE = ReadInt32(info, false);

    uint32 sizeLE = (uint32)((sizeBE << 24) | ((sizeBE << 8) & 0x00FF0000) | ((sizeBE >> 8) & 0x0000FF00) | (sizeBE >> 24));
    AllocateStorage((void **)buffer, sizeLE, DATASET_TMP, false);

    uint8 *cBuffer = NULL;
    AllocateStorage((void **)&cBuffer, cSize, DATASET_TMP, false);
    ReadBytes(info, cBuffer, cSize);

    uint32 newSize = Uncompress(&cBuffer, cSize, buffer, sizeLE);
    RemoveStorageEntry((void **)&cBuffer);

    return newSize;
}

inline void ClearDataFiles()
{
    // Unload file list
    for (int32 f = 0; f < DATAFILE_COUNT; ++f) {
        HASH_CLEAR_MD5(dataFileList[f].hash);
    }
}

} // namespace RSDK

#endif
