use crate::*;

extern "C" {
    pub fn LoadSpriteSheet(filename: *const i8, scope: uint8) -> uint16;
}
