#![allow(non_snake_case, non_camel_case_types, unused_parens)]
#![feature(adt_const_params)]

pub mod dev;
pub mod engine_core;
pub mod graphics;
pub mod scene;
pub mod storage;
pub mod user;

use engine_core::math::Vector2;
use scene::collision::{CollisionModes, TileCollisionModes};

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
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum bool32 {
    False = 0,
    True = 1,
}
impl Into<bool32> for bool {
    fn into(self) -> bool32 {
        match self {
            true => true32,
            false => false32,
        }
    }
}
impl Into<bool> for bool32 {
    fn into(self) -> bool {
        match self {
            bool32::False => false,
            bool32::True => true,
        }
    }
}
pub const false32: bool32 = bool32::False;
pub const true32: bool32 = bool32::True;

type color = u32;

// -------------------------
// CONSTANTS
// -------------------------

const SCREEN_XMAX: usize = 1280;
const SCREEN_YSIZE: usize = 240;
const SCREEN_CENTERY: usize = SCREEN_YSIZE / 2;

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
    tileCollisions: TileCollisionModes,
    interaction: bool32,
    onGround: bool32,
    active: uint8,
    #[cfg(feature = "version_2")]
    filter: uint8,
    direction: uint8,
    drawGroup: uint8,
    collisionLayers: uint8,
    collisionPlane: uint8,
    collisionMode: CollisionModes,
    drawFX: uint8,
    inkEffect: uint8,
    visible: uint8,
    onScreen: uint8,
}
