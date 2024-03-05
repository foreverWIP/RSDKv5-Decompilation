use std::{ffi::c_long, fs::File};

use crate::*;

use self::storage::text::{gen_hash_md5, gen_hash_md5_buf, HashMD5};

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
struct RSDKFileInfo {
    hash: HashMD5,
    size: int32,
    offset: int32,
    encrypted: uint8,
    useFileBuffer: uint8,
    packID: int32,
}

#[repr(C)]
struct RSDKContainer {
    name: [i8; 0x100],
    fileBuffer: *const uint8,
    fileCount: i32,
}

extern "C" {
    pub fn LoadFile(info: &mut FileInfo, filename: *const i8, fileMode: uint8) -> bool32;
}

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
        info.fileBuffer = info.fileBuffer.wrapping_add(count as usize);
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

#[no_mangle]
#[export_name = "ReadString"]
pub extern "C" fn read_string(info: &mut FileInfo, buffer: *mut i8) {
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
