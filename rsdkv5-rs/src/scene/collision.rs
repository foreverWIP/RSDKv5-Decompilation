use crate::*;

#[repr(C)]
enum TileCollisionModes {
    TILECOLLISION_NONE, // no tile collisions
    TILECOLLISION_DOWN, // downwards tile collisions
    #[cfg(feature = "version_u")]
    TILECOLLISION_UP, // upwards tile collisions
}

#[repr(C)]
enum CSides {
    C_NONE,
    C_TOP,
    C_LEFT,
    C_RIGHT,
    C_BOTTOM,
}

#[repr(C)]
struct CollisionSensor {
    position: Vector2,
    collided: bool32,
    angle: uint8,
}
const DEFAULT_COLLISIONSENSOR: CollisionSensor = CollisionSensor {
    position: Vector2::new(),
    collided: false32,
    angle: 0,
};

#[repr(C)]
struct Hitbox {
    left: int16,
    top: int16,
    right: int16,
    bottom: int16,
}
const DEFAULT_HITBOX: Hitbox = Hitbox {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

const DEBUG_HITBOX_COUNT: usize = 0x400;

#[repr(C)]
struct DebugHitboxInfo {
    type_: uint8,
    collision: uint8,
    entity: *mut u8,
    hitbox: Hitbox,
    pos: Vector2,
}
const DEFAULT_DEBUGHITBOXINFO: DebugHitboxInfo = DebugHitboxInfo {
    type_: 0,
    collision: 0,
    entity: std::ptr::null_mut(),
    hitbox: DEFAULT_HITBOX,
    pos: Vector2::new(),
};

#[repr(C)]
enum DebugHitboxTypes {
    H_TYPE_TOUCH,
    H_TYPE_CIRCLE,
    H_TYPE_BOX,
    H_TYPE_PLAT,
    H_TYPE_HAMMER,
}

#[no_mangle]
static mut collisionTolerance: int32 = 0;
#[cfg(feature = "version_u")]
#[no_mangle]
static mut useCollisionOffset: bool32 = false32;
#[cfg(not(feature = "version_u"))]
#[no_mangle]
static mut collisionOffset: int32 = 0;
#[no_mangle]
static mut collisionMaskAir: int32 = 0;

#[no_mangle]
static mut collisionOuter: Hitbox = DEFAULT_HITBOX;
#[no_mangle]
static mut collisionInner: Hitbox = DEFAULT_HITBOX;

#[no_mangle]
static mut collisionEntity: *mut Entity = std::ptr::null_mut();

#[no_mangle]
static mut sensors: [CollisionSensor; 6] = [DEFAULT_COLLISIONSENSOR; 6];

#[no_mangle]
static mut showHitboxes: bool32 = false32;
#[no_mangle]
static mut debugHitboxCount: int32 = 0;
#[no_mangle]
static mut debugHitboxList: [DebugHitboxInfo; DEBUG_HITBOX_COUNT] =
    [DEFAULT_DEBUGHITBOXINFO; DEBUG_HITBOX_COUNT];

cfg_if::cfg_if! {
    if #[cfg(feature = "version_u")] {
        #[no_mangle]
        static mut collisionMinimumDistance: int32  = TO_FIXED!(14);

        #[no_mangle]
        static mut lowCollisionTolerance: uint8   = 8;
        #[no_mangle]
        static mut highCollisionTolerance: uint8  = 14;

        #[no_mangle]
        static mut floorAngleTolerance: uint8  = 0x20;
        #[no_mangle]
        static mut wallAngleTolerance: uint8   = 0x20;
        #[no_mangle]
        static mut roofAngleTolerance: uint8   = 0x20;
    } else {
        const collisionMinimumDistance: i32 = 14;

        const lowCollisionTolerance: u8 =  8;
        const highCollisionTolerance: u8 = 15;

        const floorAngleTolerance: u8 = 0x20;
        const wallAngleTolerance: u8 =  0x20;
        const roofAngleTolerance: u8 =  0x20;
    }
}
