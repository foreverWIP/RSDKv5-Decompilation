use crate::*;

const LEGACY_ENTITY_COUNT: usize = 0x4A0;
const LEGACY_TEMPENTITY_START: usize = LEGACY_ENTITY_COUNT - 0x80;
const LEGACY_OBJECT_COUNT: usize = 0x100;
const LEGACY_TYPEGROUP_COUNT: usize = 0x103;

#[no_mangle]
pub static mut Legacy_playerListPos: int32 = 0;
