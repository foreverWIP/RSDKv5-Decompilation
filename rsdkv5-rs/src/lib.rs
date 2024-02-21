pub mod dev;
pub mod engine_core;
pub mod scene;
pub mod storage;

use std::mem::size_of;

use engine_core::math::Vector2;

// -------------------------
// STANDARD TYPES
// -------------------------

type int8 = i8;
type uint8 = u8;
type int16 = i16;
type uint16 = u16;
type int32 = i32;
type uint32 = u32;
type float = f32;

#[repr(C)]
#[derive(Clone, Copy)]
enum bool32 {
    False = 0,
    True = 1,
}
impl From<bool32> for bool {
    fn from(value: bool32) -> Self {
        match value {
            bool32::False => false,
            bool32::True => true,
        }
    }
}

type color = u32;

// -------------------------
// STRUCTS
// -------------------------

#[repr(C)]
pub struct Object {
    classID: uint16,
    active: uint8,
}

#[repr(C)]
pub struct Entity {
    #[cfg(feature = "version_u")]
    vfTable: *mut u8, // used for languages such as beeflang that always have vfTables in classes
    position: Vector2,
    scale: Vector2,
    velocity: Vector2,
    updateRange: Vector2,
    angle: int32,
    alpha: int32,
    rotation: int32,
    groundVel: int32,
    zdepth: int32,
    group: uint16,
    classID: uint16,
    inRange: bool32,
    isPermanent: bool32,
    tileCollisions: int32,
    interaction: bool32,
    onGround: bool32,
    active: uint8,
    #[cfg(feature = "version_2")]
    filter: uint8,
    direction: uint8,
    drawGroup: uint8,
    collisionLayers: uint8,
    collisionPlane: uint8,
    collisionMode: uint8,
    drawFX: uint8,
    inkEffect: uint8,
    visible: uint8,
    onScreen: uint8,
}
