pub mod legacy;
pub mod text;

use std::collections::HashMap;

use crate::*;

use self::engine_core::reader::{dataPackCount, dataPacks};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum StorageDataSets {
    DATASET_STG = 0,
    DATASET_MUS = 1,
    DATASET_SFX = 2,
    DATASET_STR = 3,
    DATASET_TMP = 4,
    DATASET_MAX, // used to signify limits
}
impl From<usize> for StorageDataSets {
    fn from(value: usize) -> Self {
        match value {
            0 => StorageDataSets::DATASET_STG,
            1 => StorageDataSets::DATASET_MUS,
            2 => StorageDataSets::DATASET_SFX,
            3 => StorageDataSets::DATASET_STR,
            4 => StorageDataSets::DATASET_TMP,
            _ => unreachable!(),
        }
    }
}

static mut DATA_STORAGE_NEW: Vec<HashMap<*mut *mut u8, Vec<u8>>> = Vec::new();

const fn get_storage_limit(set: StorageDataSets) -> usize {
    match set {
        StorageDataSets::DATASET_STG => 24 * 1024 * 1024, // 24MB
        StorageDataSets::DATASET_MUS => 8 * 1024 * 1024,  //  8MB
        StorageDataSets::DATASET_SFX => 32 * 1024 * 1024, // 32MB
        StorageDataSets::DATASET_STR => 2 * 1024 * 1024,  //  2MB
        StorageDataSets::DATASET_TMP => 8 * 1024 * 1024,  //  8MB
        _ => unreachable!(),
    }
}

#[no_mangle]
#[export_name = "InitStorage"]
pub extern "C" fn init_storage() -> bool32 {
    unsafe {
        DATA_STORAGE_NEW.clear();
        for _ in 0..(StorageDataSets::DATASET_MAX as usize) {
            DATA_STORAGE_NEW.push(HashMap::new());
        }
    }
    return true32;
}

#[no_mangle]
#[export_name = "ReleaseStorage"]
pub extern "C" fn release_storage() {
    unsafe {
        // this code isn't in steam executable, since it omits the "load datapack into memory" feature.
        // I don't think it's in the console versions either, but this never seems to be freed in those versions.
        // so, I figured doing it here would be the neatest.
        for p in 0..(dataPackCount as usize) {
            dataPacks[p].fileBuffer = None;
        }
    }
}

#[no_mangle]
#[export_name = "ClearStorage"]
pub extern "C" fn clear_storage(dataSet: StorageDataSets) {
    unsafe {
        DATA_STORAGE_NEW[dataSet as usize].clear();
    }
}

#[no_mangle]
#[export_name = "AllocateStorage"]
pub extern "C" fn allocate_storage(
    dataPtr: *mut *mut u8,
    size: uint32,
    dataSet: StorageDataSets,
    _clear: bool32,
) {
    unsafe {
        let mut new_vec_to_use = Vec::new();
        let vec_to_use = match DATA_STORAGE_NEW[dataSet as usize].get_mut(&dataPtr) {
            Some(v) => v,
            None => &mut new_vec_to_use,
        };

        if size as usize > vec_to_use.len() {
            if get_used_storage(dataSet) - vec_to_use.len() + size as usize
                >= get_storage_limit(dataSet)
            {
                garbage_collect_storage(dataSet);
            }
            vec_to_use.resize(size as usize, 0);
        }
        *dataPtr = (*vec_to_use).as_mut_ptr();
        if (*vec_to_use).as_ptr() == new_vec_to_use.as_ptr() {
            DATA_STORAGE_NEW[dataSet as usize].insert(dataPtr, new_vec_to_use);
        }
    }
}

#[no_mangle]
#[export_name = "GetUsedStorageNormalized"]
pub extern "C" fn get_used_storage_normalized(dataSet: StorageDataSets) -> f32 {
    return get_used_storage(dataSet) as f32 / get_storage_limit(dataSet) as f32;
}

fn get_used_storage(dataSet: StorageDataSets) -> usize {
    unsafe {
        DATA_STORAGE_NEW[dataSet as usize]
            .iter()
            .map(|kv| kv.1.len())
            .sum()
    }
}

#[no_mangle]
#[export_name = "RemoveStorageEntry"]
pub extern "C" fn remove_storage_entry(dataPtr: *mut *mut u8) {
    unsafe {
        if let Some(set_position) = DATA_STORAGE_NEW
            .iter()
            .position(|s| s.contains_key(&dataPtr))
        {
            DATA_STORAGE_NEW[set_position].remove(&dataPtr);
        }
    }
}

#[no_mangle]
#[export_name = "CopyStorage"]
pub extern "C" fn copy_storage(dst_managed: *mut *mut uint32, src: *mut *mut uint32, src_len: u32) {
    unsafe {
        let dst_managed = dst_managed as *mut *mut u8;
        let src = src as *mut *mut u8;
        if let Some(dst_set_position) = DATA_STORAGE_NEW
            .iter()
            .position(|s| s.contains_key(&(dst_managed as *mut *mut u8)))
        {
            let dst_set = &mut DATA_STORAGE_NEW[dst_set_position];
            let dst_vec = dst_set.get_mut(&(dst_managed as *mut *mut u8)).unwrap();
            if let Some(src_set_position) = DATA_STORAGE_NEW
                .iter()
                .position(|s| s.contains_key(&(src as *mut *mut u8)))
            {
                let src_set = &mut DATA_STORAGE_NEW[src_set_position];
                let src_vec = src_set.get(&(src as *mut *mut u8)).unwrap();
                if src_vec.len() as usize > dst_vec.len() {
                    if get_used_storage(dst_set_position.into()) as usize - dst_vec.len()
                        + src_vec.len() as usize
                        > get_storage_limit(dst_set_position.into())
                    {
                        garbage_collect_storage(dst_set_position.into());
                    }
                    dst_vec.resize(src_vec.len(), 0);
                    *dst_managed = dst_vec.as_mut_ptr();
                }
                dst_vec[..src_vec.len()].copy_from_slice(src_vec);
            } else {
                if src_len as usize > dst_vec.len() {
                    if get_used_storage(dst_set_position.into()) as usize - dst_vec.len()
                        + src_len as usize
                        > get_storage_limit(dst_set_position.into())
                    {
                        garbage_collect_storage(dst_set_position.into());
                    }
                    dst_vec.resize(src_len as usize, 0);
                    *dst_managed = dst_vec.as_mut_ptr();
                }
                (*src).copy_to(dst_vec.as_mut_ptr(), src_len as usize);
            }
        }
    }
}

#[no_mangle]
#[export_name = "GarbageCollectStorage"]
pub extern "C" fn garbage_collect_storage(set: StorageDataSets) {
    unsafe {
        DATA_STORAGE_NEW[set as usize].retain(|k, v| **k == (*v).as_mut_ptr());
    }
}

// This defragments the storage, leaving all empty space at the end.
#[no_mangle]
#[export_name = "DefragmentAndGarbageCollectStorage"]
pub extern "C" fn defragment_and_gc_storage(set: StorageDataSets) {
    garbage_collect_storage(set);
}
