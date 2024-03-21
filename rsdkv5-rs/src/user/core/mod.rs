use crate::*;

pub mod user_storage;

cfg_if::cfg_if! {
    if #[cfg(feature = "version_2")] {
        #[repr(C)]
        pub struct SKUInfo {
            platform: int32,
            language: int32,
            region: int32,
        }

        #[repr(C)]
        pub struct UnknownInfo {
            unknown1: int32,
            unknown2: int32,
            unknown3: int32,
            unknown4: int32,
            pausePress: bool32,
            unknown5: int32,
            unknown6: int32,
            unknown7: int32,
            unknown8: int32,
            unknown9: int32,
            anyKeyPress: bool32,
            unknown10: int32,
        }
    }
}
