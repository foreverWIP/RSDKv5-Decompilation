use std::{
    ffi::{c_long, c_ulong, CStr, CString},
    fs::File,
};

use miniz_oxide_c_api::mz_uncompress;

use crate::*;

use self::{
    dev::debug::{PrintLog, PrintModes},
    storage::{
        allocate_storage, remove_storage_entry,
        text::{gen_hash_md5, gen_hash_md5_buf, HashMD5},
        StorageDataSets,
    },
    user::core::user_storage::SKU_userFileDir,
};

const RSDK_SIGNATURE_RSDK: u32 = 0x4B445352; // "RSDK"
#[cfg(feature = "version_u")]
const RSDK_SIGNATURE_DATA: u32 = 0x61746144; // "Data"

const DATAFILE_COUNT: usize = 0x1000;
const DATAPACK_COUNT: usize = 4;

#[repr(C)]
pub enum Scopes {
    SCOPE_NONE,
    SCOPE_GLOBAL,
    SCOPE_STAGE,
}

#[repr(C)]
pub enum FileModes {
    FMODE_NONE,
    FMODE_RB,
    FMODE_WB,
    FMODE_RB_PLUS,
}

pub type FileIO = *const u8;

extern "C" {
    fn fread(buffer: *mut u8, size: usize, count: usize, stream: *const FileIO) -> usize;
    fn fseek(file: *const FileIO, offset: c_long, origin: i32) -> i32;
    fn fclose(file: *const FileIO) -> i32;
}

fn fSeek(file: *const FileIO, offset: c_long, whence: i32) -> i32 {
    unsafe { fseek(file, offset, whence) }
}

fn fRead(buffer: *mut u8, elementSize: usize, elementCount: usize, file: *const FileIO) -> usize {
    unsafe { fread(buffer, elementSize, elementCount, file) }
}

fn fClose(file: *const FileIO) -> i32 {
    unsafe { fclose(file) }
}

#[repr(C)]
pub struct FileInfo {
    fileSize: int32,
    externalFile: bool32,
    file: *const FileIO,
    fileBuffer: *const uint8,
    readPos: int32,
    fileOffset: int32,
    usingFileBuffer: bool,
    encrypted: bool,
    eNybbleSwap: bool,
    encryptionKeyA: [uint8; 0x10],
    encryptionKeyB: [uint8; 0x10],
    eKeyPosA: uint8,
    eKeyPosB: uint8,
    eKeyNo: uint8,
}
pub const DEFAULT_FILEINFO: FileInfo = FileInfo {
    fileSize: 0,
    externalFile: false32,
    file: std::ptr::null(),
    fileBuffer: std::ptr::null(),
    readPos: 0,
    fileOffset: 0,
    usingFileBuffer: false,
    encrypted: false,
    eNybbleSwap: false,
    encryptionKeyA: [0; 0x10],
    encryptionKeyB: [0; 0x10],
    eKeyPosA: 0,
    eKeyPosB: 0,
    eKeyNo: 0,
};

#[repr(C)]
pub struct RSDKFileInfo {
    hash: HashMD5,
    size: int32,
    offset: int32,
    encrypted: uint8,
    useFileBuffer: uint8,
    packID: int32,
}
const DEFAULT_RSDKFILEINFO: RSDKFileInfo = RSDKFileInfo {
    hash: [0; 4],
    size: 0,
    offset: 0,
    encrypted: 0,
    useFileBuffer: 0,
    packID: 0,
};

#[repr(C)]
pub struct RSDKContainer {
    pub name: [i8; 0x100],
    pub fileBuffer: Option<Box<[u8]>>,
    pub fileCount: i32,
}
const DEFAULT_RSDKCONTAINER: RSDKContainer = RSDKContainer {
    name: [0; 0x100],
    fileBuffer: None,
    fileCount: 0,
};

extern "C" {
    fn fopen(filename: *const i8, mode: *const i8) -> *mut FileIO;
    fn ftell(stream: *mut FileIO) -> u32;

    fn LoadFile_HandleMods(info: &mut FileInfo, filename: *const i8, fullFilePath: *const i8);
}

fn fOpen(filename: *const i8, mode: &str) -> *mut FileIO {
    unsafe { fopen(filename, mode.as_ptr() as *const i8) }
}

fn fTell(stream: *mut FileIO) -> u32 {
    unsafe { ftell(stream) }
}

#[no_mangle]
pub static mut dataFileList: [RSDKFileInfo; DATAFILE_COUNT] =
    [DEFAULT_RSDKFILEINFO; DATAFILE_COUNT];
#[no_mangle]
pub static mut dataPacks: [RSDKContainer; DATAPACK_COUNT] = [DEFAULT_RSDKCONTAINER; DATAPACK_COUNT];

#[no_mangle]
pub static mut dataPackCount: uint8 = 0;
#[no_mangle]
pub static mut dataFileListCount: uint16 = 0;

#[no_mangle]
pub static mut gameLogicName: [i8; 0x200] = [0; 0x200];

#[no_mangle]
pub static mut useDataPack: bool32 = false32;

const openModes: [&str; 3] = ["rb\0", "wb\0", "rb+\0"];

#[no_mangle]
#[export_name = "InitFileInfo"]
pub extern "C" fn init_file_info(info: &mut FileInfo) {
    info.file = std::ptr::null();
    info.fileSize = 0;
    info.externalFile = false32;
    info.usingFileBuffer = false;
    info.encrypted = false;
    info.readPos = 0;
    info.fileOffset = 0;
}

#[no_mangle]
#[export_name = "LoadDataPack"]
pub extern "C" fn load_data_pack(
    filePath: *const i8,
    fileOffset: usize,
    useBuffer: bool32,
) -> bool32 {
    unsafe {
        dataPacks[dataPackCount as usize] = DEFAULT_RSDKCONTAINER;
        useDataPack = false32;
        let mut info: FileInfo = DEFAULT_FILEINFO;

        let mut dataPackPath = [0u8; 0x100];

        let fullFilePathStr = CStr::from_ptr(SKU_userFileDir.as_ptr() as *const i8)
            .to_str()
            .unwrap()
            .to_owned()
            + CStr::from_ptr(filePath).to_str().unwrap();
        dataPackPath[..(fullFilePathStr.len())].copy_from_slice(fullFilePathStr.as_bytes());

        init_file_info(&mut info);
        info.externalFile = true32;
        if (load_file(
            &mut info,
            dataPackPath.as_ptr() as *const i8,
            FileModes::FMODE_RB as u8,
        ) == true32)
        {
            let sig: uint32 = read_int_32(&mut info, false32) as u32;
            if (sig != RSDK_SIGNATURE_RSDK) {
                return false32;
            }

            useDataPack = true32;

            read_int_8(&mut info); // 'v'
            read_int_8(&mut info); // version

            dataPackPath.as_ptr().copy_to(
                dataPacks[dataPackCount as usize].name.as_ptr() as *mut u8,
                dataPackPath.len(),
            );

            dataPacks[dataPackCount as usize].fileCount = read_int_16(&mut info) as i32;
            for f in 0..dataPacks[dataPackCount as usize].fileCount {
                let mut b = [0u8; 4];
                for y in 0..4 {
                    read_bytes(&mut info, b.as_mut_ptr(), 4);
                    dataFileList[f as usize].hash[y] = ((b[0] as u32) << 24)
                        | ((b[1] as u32) << 16)
                        | ((b[2] as u32) << 8)
                        | ((b[3] as u32) << 0);
                }

                dataFileList[f as usize].offset = read_int_32(&mut info, false32);
                dataFileList[f as usize].size = read_int_32(&mut info, false32);

                dataFileList[f as usize].encrypted =
                    if ((dataFileList[f as usize].size as u32) & 0x80000000) != 0 {
                        1
                    } else {
                        0
                    };
                dataFileList[f as usize].size &= 0x7FFFFFFF;
                dataFileList[f as usize].useFileBuffer = useBuffer as u8;
                dataFileList[f as usize].packID = dataPackCount as i32;
            }

            dataPacks[dataPackCount as usize].fileBuffer = None;
            if (useBuffer == true32) {
                let fileSize = info.fileSize;
                dataPacks[dataPackCount as usize].fileBuffer =
                    Some(vec![0; fileSize as usize].into_boxed_slice());
                seek_set(&mut info, 0);
                let mut buf = vec![0u8; fileSize as usize].into_boxed_slice();
                read_bytes(&mut info, buf.as_mut_ptr(), fileSize);
                dataPacks[dataPackCount as usize].fileBuffer = Some(buf);
            }

            dataFileListCount += dataPacks[dataPackCount as usize].fileCount as u16;
            dataPackCount += 1;

            close_file(&mut info);

            return true32;
        } else {
            useDataPack = false32;
            return false32;
        }
    }
}

#[no_mangle]
#[export_name = "OpenDataFile"]
pub extern "C" fn open_data_file(info: &mut FileInfo, filename: *const i8) -> bool32 {
    unsafe {
        let hash = gen_hash_md5(
            &CStr::from_ptr(filename)
                .to_str()
                .unwrap()
                .to_owned()
                .to_ascii_lowercase(),
        );

        for f in 0..dataFileListCount {
            let file = &dataFileList[f as usize];

            if (hash != file.hash) {
                continue;
            }

            info.usingFileBuffer = file.useFileBuffer != 0;
            if (file.useFileBuffer == 0) {
                info.file = fOpen(dataPacks[file.packID as usize].name.as_ptr(), "rb\0");
                if (info.file.is_null()) {
                    PrintLog(
                        PrintModes::PRINT_NORMAL,
                        "File not found (Unable to open datapack): %s".as_ptr() as *const i8,
                        filename,
                    );
                    return false32;
                }

                fSeek(info.file, file.offset, 0);
            } else {
                // a bit of a hack, but it is how it is in the original
                info.file = &dataPacks[file.packID as usize]
                    .fileBuffer
                    .as_ref()
                    .unwrap()
                    .as_ptr()
                    .wrapping_add(file.offset as usize);

                let fileBuffer = info.file as *const u8;
                info.fileBuffer = fileBuffer;
            }

            info.fileSize = file.size;
            info.readPos = 0;
            info.fileOffset = file.offset;
            info.encrypted = file.encrypted != 0;
            info.encryptionKeyA.fill(0);
            info.encryptionKeyB.fill(0);
            if (info.encrypted) {
                generate_e_load_keys(info, filename, info.fileSize);
                info.eKeyNo = ((info.fileSize / 4) & 0x7F) as u8;
                info.eKeyPosA = 0;
                info.eKeyPosB = 8;
                info.eNybbleSwap = false;
            }

            PrintLog(
                PrintModes::PRINT_NORMAL,
                "Loaded data file %s".as_ptr() as *const i8,
                filename,
            );
            return true32;
        }

        PrintLog(
            PrintModes::PRINT_NORMAL,
            "Data file not found: %s".as_ptr() as *const i8,
            filename,
        );
        return false32;
    }
}

#[no_mangle]
#[export_name = "LoadFile"]
pub extern "C" fn load_file(info: &mut FileInfo, filename: *const i8, fileMode: u8) -> bool32 {
    if (!info.file.is_null()) {
        return false32;
    }

    unsafe {
        let mut fullFilePath = CStr::from_ptr(filename).to_str().unwrap().to_owned() + "\0";

        if cfg!(feature = "mod_loader") {
            LoadFile_HandleMods(info, filename, fullFilePath.as_ptr() as *const i8);
        }

        // somewhat hacky but also pleases the mod gods
        if (info.externalFile == false32) {
            fullFilePath = CStr::from_ptr(SKU_userFileDir.as_ptr() as *const i8)
                .to_str()
                .unwrap()
                .to_owned()
                + &fullFilePath;
        }

        if (info.externalFile == false32
            && fileMode == FileModes::FMODE_RB as u8
            && useDataPack.into())
        {
            return open_data_file(info, filename);
        }

        if (fileMode == FileModes::FMODE_RB as u8
            || fileMode == FileModes::FMODE_WB as u8
            || fileMode == FileModes::FMODE_RB_PLUS as u8)
        {
            info.file = fOpen(
                fullFilePath.as_ptr() as *const i8,
                openModes[fileMode as usize - 1],
            );
        }

        if (info.file.is_null()) {
            PrintLog(
                PrintModes::PRINT_NORMAL,
                "File not found: %s".as_ptr() as *const i8,
                fullFilePath.as_ptr(),
            );
            return false32;
        }

        info.readPos = 0;
        info.fileSize = 0;

        if (fileMode != FileModes::FMODE_WB as u8) {
            fSeek(info.file, 0, 2);
            info.fileSize = fTell(info.file as *mut *const u8) as i32;
            fSeek(info.file, 0, 0);
        }
        PrintLog(
            PrintModes::PRINT_NORMAL,
            "Loaded file %s".as_ptr() as *const i8,
            fullFilePath.as_ptr(),
        );
        return true32;
    }
}

#[no_mangle]
#[export_name = "CloseFile"]
pub extern "C" fn close_file(info: &mut FileInfo) {
    if (!info.usingFileBuffer && !info.file.is_null()) {
        fClose(info.file);
    }

    info.file = std::ptr::null();
}

#[no_mangle]
#[export_name = "SkipBytes"]
pub extern "C" fn skip_bytes(info: &mut FileInfo, mut size: i32) {
    if (size != 0) {
        while (size > 0) {
            info.eKeyPosA += 1;
            info.eKeyPosB += 1;

            if (info.eKeyPosA <= 15) {
                if (info.eKeyPosB > 12) {
                    info.eKeyPosB = 0;
                    info.eNybbleSwap = !info.eNybbleSwap;
                }
            } else if (info.eKeyPosB <= 8) {
                info.eKeyPosA = 0;
                info.eNybbleSwap = !info.eNybbleSwap;
            } else {
                info.eKeyNo += 2;
                info.eKeyNo &= 0x7F;

                if (info.eNybbleSwap) {
                    info.eNybbleSwap = false;

                    info.eKeyPosA = info.eKeyNo % 7;
                    info.eKeyPosB = (info.eKeyNo % 12) + 2;
                } else {
                    info.eNybbleSwap = true;

                    info.eKeyPosA = (info.eKeyNo % 12) + 3;
                    info.eKeyPosB = info.eKeyNo % 7;
                }
            }

            size -= 1;
        }
    }
}

#[no_mangle]
#[export_name = "Seek_Set"]
pub extern "C" fn seek_set(info: &mut FileInfo, count: i32) {
    if (info.readPos != count) {
        if (info.encrypted) {
            info.eKeyNo = ((info.fileSize / 4) & 0x7F) as u8;
            info.eKeyPosA = 0;
            info.eKeyPosB = 8;
            info.eNybbleSwap = false;
            skip_bytes(info, count);
        }

        info.readPos = count;
        if (info.usingFileBuffer) {
            let fileBuffer = info.file as *const u8;
            info.fileBuffer = fileBuffer.wrapping_add(info.readPos as usize);
        } else {
            fSeek(info.file, info.fileOffset + info.readPos, 0);
        }
    }
}

#[no_mangle]
#[export_name = "Seek_Cur"]
pub extern "C" fn seek_cur(info: &mut FileInfo, count: i32) {
    info.readPos += count;

    if (info.encrypted) {
        skip_bytes(info, count);
    }

    if (info.usingFileBuffer) {
        info.fileBuffer = info.fileBuffer.wrapping_add((count as isize) as usize);
    } else {
        fSeek(info.file, count, 1); // if this works i'll blow up
    }
}

#[no_mangle]
#[export_name = "ReadBytes"]
pub extern "C" fn read_bytes(info: &mut FileInfo, data: *mut u8, count: i32) -> usize {
    let mut bytesRead: usize = 0;

    if (info.usingFileBuffer) {
        bytesRead = count.min(info.fileSize - info.readPos) as usize;
        unsafe {
            data.copy_from(info.fileBuffer, bytesRead);
        }
        info.fileBuffer = info.fileBuffer.wrapping_add(bytesRead);
    } else {
        bytesRead = fRead(data, 1, count as usize, info.file);
    }

    if (info.encrypted) {
        decrypt_bytes(info, data, bytesRead);
    }

    info.readPos += bytesRead as i32;
    return bytesRead;
}

#[no_mangle]
#[export_name = "ReadInt8"]
pub extern "C" fn read_int_8(info: &mut FileInfo) -> uint8 {
    let mut buf = 0u8;
    read_bytes(info, (&mut buf) as *mut u8, 1);
    buf
}

#[no_mangle]
#[export_name = "ReadInt16"]
pub extern "C" fn read_int_16(info: &mut FileInfo) -> int16 {
    let mut buf = 0i16;
    read_bytes(info, (&mut buf) as *mut i16 as *mut u8, 2);
    if cfg!(target_endian = "big") {
        buf.swap_bytes()
    } else {
        buf
    }
}

#[no_mangle]
#[export_name = "ReadInt32"]
pub extern "C" fn read_int_32(info: &mut FileInfo, swap_endian: bool32) -> int32 {
    let mut buf = 0i32;
    read_bytes(info, (&mut buf) as *mut i32 as *mut u8, 4);
    if swap_endian == true32 {
        buf = buf.swap_bytes();
    }
    if cfg!(target_endian = "big") {
        buf.swap_bytes()
    } else {
        buf
    }
}

#[no_mangle]
#[export_name = "ReadInt64"]
pub extern "C" fn read_int_64(info: &mut FileInfo) -> i64 {
    let mut buf = 0i64;
    read_bytes(info, (&mut buf) as *mut i64 as *mut u8, 8);
    if cfg!(target_endian = "big") {
        buf.swap_bytes()
    } else {
        buf
    }
}

#[no_mangle]
#[export_name = "ReadSingle"]
pub extern "C" fn read_single(info: &mut FileInfo) -> f32 {
    let mut buf = 0i32;
    read_bytes(info, (&mut buf) as *mut i32 as *mut u8, 4);
    if cfg!(target_endian = "big") {
        f32::from_be_bytes(buf.to_be_bytes())
    } else {
        f32::from_le_bytes(buf.to_le_bytes())
    }
}

pub fn read_string(info: &mut FileInfo) -> String {
    let size = read_int_8(info);
    let mut ret = String::from("");
    for _ in 0..size {
        ret += &(read_int_8(info) as char).to_string();
    }
    ret
}

#[no_mangle]
#[export_name = "ReadString"]
pub extern "C" fn read_string_buf(info: &mut FileInfo, buffer: *mut i8) {
    let size = read_int_8(info);
    read_bytes(info, buffer as *mut u8, size as i32);
    unsafe {
        *buffer.wrapping_add(size as usize) = 0;
    }
}

#[no_mangle]
#[export_name = "DecryptBytes"]
pub extern "C" fn decrypt_bytes(info: &mut FileInfo, buffer: *mut u8, mut size: usize) {
    if (size != 0) {
        let mut data = buffer;
        unsafe {
            while (size > 0) {
                *data ^= info.eKeyNo ^ info.encryptionKeyB[info.eKeyPosB as usize];
                if (info.eNybbleSwap) {
                    *data = ((*data << 4) + (*data >> 4)) & 0xFF;
                }
                *data ^= info.encryptionKeyA[info.eKeyPosA as usize];

                info.eKeyPosA += 1;
                info.eKeyPosB += 1;

                if (info.eKeyPosA <= 15) {
                    if (info.eKeyPosB > 12) {
                        info.eKeyPosB = 0;
                        info.eNybbleSwap = !info.eNybbleSwap;
                    }
                } else if (info.eKeyPosB <= 8) {
                    info.eKeyPosA = 0;
                    info.eNybbleSwap = !info.eNybbleSwap;
                } else {
                    info.eKeyNo += 2;
                    info.eKeyNo &= 0x7F;

                    if (info.eNybbleSwap) {
                        info.eNybbleSwap = false;

                        info.eKeyPosA = info.eKeyNo % 7;
                        info.eKeyPosB = (info.eKeyNo % 12) + 2;
                    } else {
                        info.eNybbleSwap = true;

                        info.eKeyPosA = (info.eKeyNo % 12) + 3;
                        info.eKeyPosB = info.eKeyNo % 7;
                    }
                }

                data = data.wrapping_add(1);
                size -= 1;
            }
        }
    }
}

#[no_mangle]
#[export_name = "GenerateELoadKeys"]
pub extern "C" fn generate_e_load_keys(info: &mut FileInfo, key1: *const i8, key2: i32) {
    let c_str = unsafe {
        assert!(!key1.is_null());

        std::ffi::CStr::from_ptr(key1).to_str().unwrap()
    };

    // KeyA
    let hash = gen_hash_md5(&c_str.to_ascii_uppercase());

    for i in 0..4 {
        for j in 0..4 {
            info.encryptionKeyA[i * 4 + j] = ((hash[i] >> (8 * (j ^ 3))) & 0xFF) as u8;
        }
    }

    // KeyB
    let hash = gen_hash_md5(&key2.to_string());

    for i in 0..4 {
        for j in 0..4 {
            info.encryptionKeyB[i * 4 + j] = ((hash[i] >> (8 * (j ^ 3))) & 0xFF) as u8;
        }
    }
}

#[no_mangle]
#[export_name = "Uncompress"]
pub extern "C" fn uncompress(
    cBuffer: *mut *mut uint8,
    cSize: int32,
    buffer: *mut *mut u8,
    size: int32,
) -> int32 {
    if (buffer.is_null() || cBuffer.is_null()) {
        return 0;
    }

    let cLen: c_ulong = cSize as c_ulong;
    let mut destLen: c_ulong = size as c_ulong;

    unsafe {
        _ = mz_uncompress(*buffer, (&mut destLen) as *mut c_ulong, *cBuffer, cLen);
    }

    return destLen as i32;
}

// The buffer passed in parameter is allocated here, so it's up to the caller to free it once it goes unused
pub fn read_compressed(info: &mut FileInfo) -> Vec<u8> {
    let cSize: uint32 = (read_int_32(info, false32) - 4) as u32;
    let sizeBE: uint32 = read_int_32(info, false32) as u32;

    let sizeLE: uint32 = ((sizeBE << 24)
        | ((sizeBE << 8) & 0x00FF0000)
        | ((sizeBE >> 8) & 0x0000FF00)
        | (sizeBE >> 24)) as u32;
    let mut ret = Vec::new();
    ret.resize(sizeLE as usize, 0);

    let mut cBuffer = Vec::new();
    cBuffer.resize(cSize as usize, 0);
    read_bytes(info, cBuffer.as_mut_ptr(), cSize as i32);

    let newSize: uint32 = uncompress(
        &mut cBuffer.as_mut_ptr(),
        cSize as i32,
        &mut ret.as_mut_ptr(),
        sizeLE as i32,
    ) as u32;

    return ret;
}

#[no_mangle]
#[export_name = "ClearDataFiles"]
pub extern "C" fn clear_data_files() {
    unsafe {
        // Unload file list
        for f in 0..DATAFILE_COUNT {
            dataFileList[f].hash = HashMD5::default();
        }
    }
}
