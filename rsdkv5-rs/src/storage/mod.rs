pub mod legacy;
pub mod text;

use crate::*;

use self::engine_core::reader::{dataPackCount, dataPacks};

// Macro to access the header variables of a block of memory.
// Note that this is pointless if the pointer is already pointing directly at the header rather than the memory after it.
// #define HEADER(memory, header_value) memory[-HEADER_SIZE + header_value]
macro_rules! HEADER {
    ($memory:expr, $header_value:expr) => {
        (*$memory.wrapping_add(
            (-(HeaderFields::HEADER_SIZE as isize)) as usize + $header_value as usize,
        )) as usize
    };
}

// Every block of allocated memory is prefixed with a header that consists of the following four longwords.
#[repr(C)]
enum HeaderFields {
    // Whether the block of memory is actually allocated or not.
    HEADER_ACTIVE,
    // Which 'data set' this block of memory belongs to.
    HEADER_SET_ID,
    // The offset in the buffer which the block of memory begins at.
    HEADER_DATA_OFFSET,
    // How long the block of memory is (measured in 'uint32's).
    HEADER_DATA_LENGTH,
    // This is not part of the header: it's just a bit of enum magic to calculate the size of the header.
    HEADER_SIZE,
}

const STORAGE_ENTRY_COUNT: usize = 0x1000;

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

#[repr(C)]
struct DataStorage {
    memoryTable: *mut uint32,
    usedStorage: uint32,
    storageLimit: uint32,
    dataEntries: [*mut *mut uint32; STORAGE_ENTRY_COUNT], // pointer to the actual variable
    storageEntries: [*mut uint32; STORAGE_ENTRY_COUNT],   // pointer to the storage in "memoryTable"
    entryCount: uint32,
    clearCount: uint32,
}
const DEFAULT_DATASTORAGE: DataStorage = DataStorage {
    memoryTable: std::ptr::null_mut(),
    usedStorage: 0,
    storageLimit: 0,
    dataEntries: [std::ptr::null_mut(); STORAGE_ENTRY_COUNT],
    storageEntries: [std::ptr::null_mut(); STORAGE_ENTRY_COUNT],
    entryCount: 0,
    clearCount: 0,
};

#[no_mangle]
static mut dataStorage: [DataStorage; StorageDataSets::DATASET_MAX as usize] =
    [DEFAULT_DATASTORAGE; StorageDataSets::DATASET_MAX as usize];

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

#[no_mangle]
#[export_name = "InitStorage"]
pub extern "C" fn init_storage() -> bool32 {
    unsafe {
        // Storage limits.
        dataStorage[StorageDataSets::DATASET_STG as usize].storageLimit = 24 * 1024 * 1024; // 24MB
        dataStorage[StorageDataSets::DATASET_MUS as usize].storageLimit = 8 * 1024 * 1024; //  8MB
        dataStorage[StorageDataSets::DATASET_SFX as usize].storageLimit = 32 * 1024 * 1024; // 32MB
        dataStorage[StorageDataSets::DATASET_STR as usize].storageLimit = 2 * 1024 * 1024; //  2MB
        dataStorage[StorageDataSets::DATASET_TMP as usize].storageLimit = 8 * 1024 * 1024; //  8MB

        for s in 0..(StorageDataSets::DATASET_MAX as usize) {
            dataStorage[s].usedStorage = 0;
            dataStorage[s].entryCount = 0;
            dataStorage[s].clearCount = 0;
            dataStorage[s].memoryTable = malloc(dataStorage[s].storageLimit as usize) as *mut u32;

            if (dataStorage[s].memoryTable.is_null()) {
                return false32;
            }
        }
    }

    return true32;
}

#[no_mangle]
#[export_name = "ReleaseStorage"]
pub extern "C" fn release_storage() {
    unsafe {
        for s in 0..(StorageDataSets::DATASET_MAX as usize) {
            if (!dataStorage[s].memoryTable.is_null()) {
                free(dataStorage[s].memoryTable as *mut u8);
            }

            dataStorage[s].usedStorage = 0;
            dataStorage[s].entryCount = 0;
            dataStorage[s].clearCount = 0;
        }

        // this code isn't in steam executable, since it omits the "load datapack into memory" feature.
        // I don't think it's in the console versions either, but this never seems to be freed in those versions.
        // so, I figured doing it here would be the neatest.
        for p in 0..(dataPackCount as usize) {
            if (!dataPacks[p].fileBuffer.is_null()) {
                free(dataPacks[p].fileBuffer as *mut u8);
            }

            dataPacks[p].fileBuffer = std::ptr::null();
        }
    }
}

#[no_mangle]
#[export_name = "AllocateStorage"]
pub extern "C" fn allocate_storage(
    dataPtr: *mut *mut u8,
    mut size: uint32,
    dataSet: StorageDataSets,
    clear: bool32,
) {
    unsafe {
        let data = dataPtr as *mut *mut u32;
        *data = std::ptr::null_mut();

        if (dataSet < StorageDataSets::DATASET_MAX) {
            let dataSet_usize = dataSet as usize;
            // Align allocation to prevent unaligned memory accesses later on.
            let size_aligned: uint32 = size & (-((usize::BITS / 8) as i32)) as u32;

            if (size_aligned < size) {
                size = size_aligned + (usize::BITS / 8);
            }

            if ((dataStorage[dataSet_usize].entryCount as usize) < STORAGE_ENTRY_COUNT) {
                let storage = &mut dataStorage[dataSet_usize];

                // Bug: The original release never takes into account the size of the header when checking if there's enough storage left.
                // Omitting this will overflow the memory pool when (storageLimit - usedStorage + size) < header size (16 bytes here).
                if (storage.usedStorage * (uint32::BITS / 8)
                    + size
                    + (HeaderFields::HEADER_SIZE as u32 * (uint32::BITS / 8))
                    < storage.storageLimit)
                {
                    // HEADER_ACTIVE
                    *storage
                        .memoryTable
                        .wrapping_add(storage.usedStorage as usize) = true32 as u32;
                    storage.usedStorage += 1;

                    // HEADER_SET_ID
                    *storage
                        .memoryTable
                        .wrapping_add(storage.usedStorage as usize) = dataSet_usize as u32;
                    storage.usedStorage += 1;

                    // HEADER_DATA_OFFSET
                    *storage
                        .memoryTable
                        .wrapping_add(storage.usedStorage as usize) = storage.usedStorage
                        + HeaderFields::HEADER_SIZE as u32
                        - HeaderFields::HEADER_DATA_OFFSET as u32;
                    storage.usedStorage += 1;

                    // HEADER_DATA_LENGTH
                    *storage
                        .memoryTable
                        .wrapping_add(storage.usedStorage as usize) = size;
                    storage.usedStorage += 1;

                    *data = storage
                        .memoryTable
                        .wrapping_add(storage.usedStorage as usize);
                    storage.usedStorage += size / (u32::BITS / 8);

                    dataStorage[dataSet_usize].dataEntries[storage.entryCount as usize] = data;
                    dataStorage[dataSet_usize].storageEntries[storage.entryCount as usize] = *data;

                    storage.entryCount += 1;
                } else {
                    // We've run out of room, so perform defragmentation and garbage-collection.
                    defragment_and_gc_storage(dataSet);

                    // If there is now room, then perform allocation.
                    // Yes, this really is a massive chunk of duplicate code.
                    if (storage.usedStorage * (u32::BITS / 8)
                        + size
                        + (HeaderFields::HEADER_SIZE as u32 * (u32::BITS / 8))
                        < storage.storageLimit)
                    {
                        // HEADER_ACTIVE
                        *storage
                            .memoryTable
                            .wrapping_add(storage.usedStorage as usize) = true32 as u32;
                        storage.usedStorage += 1;

                        // HEADER_SET_ID
                        *storage
                            .memoryTable
                            .wrapping_add(storage.usedStorage as usize) = dataSet_usize as u32;
                        storage.usedStorage += 1;

                        // HEADER_DATA_OFFSET
                        *storage
                            .memoryTable
                            .wrapping_add(storage.usedStorage as usize) = storage.usedStorage
                            + HeaderFields::HEADER_SIZE as u32
                            - HeaderFields::HEADER_DATA_OFFSET as u32;
                        storage.usedStorage += 1;

                        // HEADER_DATA_LENGTH
                        *storage
                            .memoryTable
                            .wrapping_add(storage.usedStorage as usize) = size;
                        storage.usedStorage += 1;

                        *data = storage
                            .memoryTable
                            .wrapping_add(storage.usedStorage as usize);
                        storage.usedStorage += size / (u32::BITS / 8);

                        dataStorage[dataSet_usize].dataEntries[storage.entryCount as usize] = data;
                        dataStorage[dataSet_usize].storageEntries[storage.entryCount as usize] =
                            *data;

                        storage.entryCount += 1;
                    }
                }

                // If there are too many storage entries, then perform garbage collection.
                if (storage.entryCount as usize >= STORAGE_ENTRY_COUNT) {
                    garbage_collect_storage(dataSet);
                }

                // Clear the allocated memory if requested.
                if (!(*data).is_null() && clear == true32) {
                    for i in 0..size {
                        *(*data as *mut u8).wrapping_add(i as usize) = 0;
                    }
                }
            }
        }
    }
}

#[no_mangle]
#[export_name = "RemoveStorageEntry"]
pub extern "C" fn remove_storage_entry(dataPtr: *mut *mut u8) {
    unsafe {
        if (!dataPtr.is_null() && !(*dataPtr).is_null()) {
            let data: *mut uint32 = *(dataPtr as *mut *mut u32);

            let mut set = HEADER!(data, HeaderFields::HEADER_SET_ID);
            for e in 0..(dataStorage[set].entryCount as usize) {
                // make sure dataEntries[e] isn't null. If it is null by some ungodly chance then it was probably already freed or something idk
                if (!dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries[e]
                    .is_null()
                    && *dataPtr
                        == *dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries[e]
                            as *mut u8)
                {
                    *dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries[e] =
                        std::ptr::null_mut();
                    dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries[e] =
                        std::ptr::null_mut();
                }

                set = HEADER!(data, HeaderFields::HEADER_SET_ID);
            }

            let mut newEntryCount = 0;
            set = HEADER!(data, HeaderFields::HEADER_SET_ID);
            for entryID in 0..(dataStorage[set].entryCount as usize) {
                if (!dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries[entryID]
                    .is_null())
                {
                    if (entryID != newEntryCount) {
                        dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries
                            [newEntryCount] = dataStorage
                            [HEADER!(data, HeaderFields::HEADER_SET_ID)]
                        .dataEntries[entryID];
                        dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries
                            [entryID] = std::ptr::null_mut();
                        dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].storageEntries
                            [newEntryCount] = dataStorage
                            [HEADER!(data, HeaderFields::HEADER_SET_ID)]
                        .storageEntries[entryID];
                        dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].storageEntries
                            [entryID] = std::ptr::null_mut();
                    }

                    newEntryCount += 1;
                }

                set = HEADER!(data, HeaderFields::HEADER_SET_ID);
            }

            dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].entryCount =
                newEntryCount as u32;

            for e in newEntryCount..STORAGE_ENTRY_COUNT {
                dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].dataEntries[e] =
                    std::ptr::null_mut();
                dataStorage[HEADER!(data, HeaderFields::HEADER_SET_ID)].storageEntries[e] =
                    std::ptr::null_mut();
            }

            *data.wrapping_add(
                (-(HeaderFields::HEADER_SIZE as isize)) as usize
                    + (HeaderFields::HEADER_ACTIVE) as usize,
            ) = false32 as u32;
        }
    }
}

#[no_mangle]
#[export_name = "CopyStorage"]
pub extern "C" fn copy_storage(src: *mut *mut uint32, dst: *mut *mut uint32) {
    if (!dst.is_null()) {
        unsafe {
            let dstPtr: *mut uint32 = *dst;
            *src = *dst;

            if ((dataStorage[HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)].entryCount as usize)
                < STORAGE_ENTRY_COUNT)
            {
                dataStorage[HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)].dataEntries[dataStorage
                    [HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)]
                .entryCount
                    as usize] = src;
                dataStorage[HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)].storageEntries[dataStorage
                    [HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)]
                .entryCount
                    as usize] = *src;

                dataStorage[HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)].entryCount += 1;

                if ((dataStorage[HEADER!(dstPtr, HeaderFields::HEADER_SET_ID)].entryCount as usize)
                    >= STORAGE_ENTRY_COUNT)
                {
                    garbage_collect_storage(StorageDataSets::from(HEADER!(
                        dstPtr,
                        HeaderFields::HEADER_SET_ID
                    )));
                }
            }
        }
    }
}

#[no_mangle]
#[export_name = "GarbageCollectStorage"]
pub extern "C" fn garbage_collect_storage(set: StorageDataSets) {
    let set = set as usize;
    if (set < StorageDataSets::DATASET_MAX as usize) {
        unsafe {
            for e in 0..(dataStorage[set].entryCount as usize) {
                // So what's happening here is the engine is checking to see if the storage entry
                // (which is the pointer to the "memoryTable" offset that is allocated for this entry)
                // matches what the actual variable that allocated the storage is currently pointing to.
                // if they don't match, the storage entry is considered invalid and marked for removal.

                if (!dataStorage[set].dataEntries[e].is_null()
                    && *dataStorage[set].dataEntries[e] != dataStorage[set].storageEntries[e])
                {
                    dataStorage[set].dataEntries[e] = std::ptr::null_mut();
                }
            }

            let mut newEntryCount = 0;
            for entryID in 0..(dataStorage[set].entryCount as usize) {
                if (!dataStorage[set].dataEntries[entryID].is_null()) {
                    if (entryID != newEntryCount) {
                        dataStorage[set].dataEntries[newEntryCount] =
                            dataStorage[set].dataEntries[entryID];
                        dataStorage[set].dataEntries[entryID] = std::ptr::null_mut();
                        dataStorage[set].storageEntries[newEntryCount] =
                            dataStorage[set].storageEntries[entryID];
                        dataStorage[set].storageEntries[entryID] = std::ptr::null_mut();
                    }

                    newEntryCount += 1;
                }
            }
            dataStorage[set].entryCount = newEntryCount as u32;

            for e in (dataStorage[set].entryCount as usize)..STORAGE_ENTRY_COUNT {
                dataStorage[set].dataEntries[e] = std::ptr::null_mut();
                dataStorage[set].storageEntries[e] = std::ptr::null_mut();
            }
        }
    }
}

// This defragments the storage, leaving all empty space at the end.
#[no_mangle]
#[export_name = "DefragmentAndGarbageCollectStorage"]
pub extern "C" fn defragment_and_gc_storage(set: StorageDataSets) {
    let mut processedStorage: uint32 = 0;
    let mut unusedStorage: uint32 = 0;

    let set_usize = set as usize;
    unsafe {
        let mut defragmentDestination: *mut uint32 = dataStorage[set_usize].memoryTable;
        let mut currentHeader: *mut uint32 = dataStorage[set_usize].memoryTable;

        dataStorage[set_usize].clearCount += 1;

        // Perform garbage-collection. This deallocates all memory allocations that are no longer being used.
        garbage_collect_storage(set);

        // This performs defragmentation. It works by removing 'gaps' between the various blocks of allocated memory,
        // grouping them all together at the start of the buffer while all the empty space goes at the end.
        // Avoiding fragmentation is important, as fragmentation can cause allocations to fail despite there being
        // enough free memory because that free memory isn't contiguous.
        while (processedStorage < dataStorage[set_usize].usedStorage) {
            let dataPtr: *mut uint32 = dataStorage[set_usize].memoryTable.wrapping_add(
                *currentHeader.wrapping_add(HeaderFields::HEADER_DATA_OFFSET as usize) as usize,
            );
            let size: uint32 = (*currentHeader
                .wrapping_add(HeaderFields::HEADER_DATA_LENGTH as usize)
                / (u32::BITS / 8))
                + HeaderFields::HEADER_SIZE as u32;

            // Check if this block of memory is currently allocated.
            *currentHeader.wrapping_add(HeaderFields::HEADER_ACTIVE as usize) = false32 as u32;

            for e in 0..(dataStorage[set_usize].entryCount as usize) {
                if (dataPtr == dataStorage[set_usize].storageEntries[e]) {
                    *currentHeader.wrapping_add(HeaderFields::HEADER_ACTIVE as usize) =
                        true32 as u32;
                }
            }

            if (*currentHeader.wrapping_add(HeaderFields::HEADER_ACTIVE as usize) != 0) {
                // This memory is being used.
                processedStorage += size;

                if (currentHeader > defragmentDestination) {
                    // This memory has a gap before it, so move it backwards into that free space.
                    for i in 0..size {
                        *defragmentDestination = *currentHeader;
                        defragmentDestination = defragmentDestination.wrapping_add(1);
                        currentHeader = currentHeader.wrapping_add(1);
                    }
                } else {
                    // This memory doesn't have a gap before it, so we don't need to move it - just skip it instead.
                    defragmentDestination = defragmentDestination.wrapping_add(size as usize);
                    currentHeader = currentHeader.wrapping_add(size as usize);
                }
            } else {
                // This memory is not being used, so skip it.
                currentHeader = currentHeader.wrapping_add(size as usize);
                processedStorage += size;
                unusedStorage += size;
            }
        }

        // If defragmentation occurred, then we need to update every single
        // pointer to allocated memory to point to their new locations in the buffer.
        if (unusedStorage != 0) {
            dataStorage[set_usize].usedStorage -= unusedStorage;

            let mut currentHeader: *mut uint32 = dataStorage[set_usize].memoryTable;

            let mut dataOffset: uint32 = 0;
            while (dataOffset < dataStorage[set_usize].usedStorage) {
                let dataPtr: *mut uint32 = dataStorage[set_usize].memoryTable.wrapping_add(
                    *currentHeader.wrapping_add(HeaderFields::HEADER_DATA_OFFSET as usize) as usize,
                );
                let size: uint32 = (*currentHeader
                    .wrapping_add(HeaderFields::HEADER_DATA_LENGTH as usize)
                    / (u32::BITS / 8))
                    + HeaderFields::HEADER_SIZE as u32; // size (in int32s)

                // Find every single pointer to this memory allocation and update them with its new address.
                for c in 0..(dataStorage[set_usize].entryCount as usize) {
                    // make sure dataEntries[e] isn't null. If it is null by some ungodly chance then it was probably already freed or something idk
                    if (dataPtr == dataStorage[set_usize].storageEntries[c]
                        && !dataStorage[set_usize].dataEntries[c].is_null())
                    {
                        let val = currentHeader.wrapping_add(HeaderFields::HEADER_SIZE as usize);
                        dataStorage[set_usize].storageEntries[c] = val;
                        *dataStorage[set_usize].dataEntries[c] = val;
                    }
                }

                // Update the offset in the allocation's header too.
                *currentHeader.wrapping_add(HeaderFields::HEADER_DATA_OFFSET as usize) =
                    dataOffset + HeaderFields::HEADER_SIZE as u32;

                // Advance to the next memory allocation.
                currentHeader = currentHeader.wrapping_add(size as usize);
                dataOffset += size;
            }
        }
    }
}
