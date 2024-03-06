use crate::*;

use self::graphics::drawing::LAYER_COUNT;

use super::{collisionMasks, tileInfo, tileLayers, TILE_SIZE};

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
pub struct CollisionSensor {
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
pub enum CollisionModes {
    CMODE_FLOOR,
    CMODE_LWALL,
    CMODE_ROOF,
    CMODE_RWALL,
}

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

#[no_mangle]
#[export_name = "ObjectTileGrip"]
pub extern "C" fn object_tile_grip(
    entity: &mut Entity,
    cLayers: uint16,
    cMode: CollisionModes,
    cPlane: uint8,
    xOffset: int32,
    yOffset: int32,
    tolerance: int32,
) -> bool32 {
    let mut layerID: int32 = 1;
    let mut collided: bool32 = false32;
    let mut posX: int32 = FROM_FIXED!(xOffset + entity.position.x);
    let mut posY: int32 = FROM_FIXED!(yOffset + entity.position.y);

    let mut solid: int32 = 0;
    unsafe {
        match cMode {
            CollisionModes::CMODE_FLOOR => {
                solid = if cPlane != 0 { (1 << 14) } else { (1 << 12) };

                // for (int32 l = 0; l < LAYER_COUNT; ++l, layerID <<= 1) {
                for l in 0..LAYER_COUNT {
                    if (cLayers & layerID as u16) != 0 {
                        let layer = &tileLayers[l];
                        let colX: int32 = posX - layer.position.x;
                        let mut colY: int32 = posY - layer.position.y;
                        let mut cy: int32 = (colY & -(TILE_SIZE as i32)) - TILE_SIZE as i32;
                        if (colX >= 0 && colX < TILE_SIZE as i32 * layer.xsize as i32) {
                            for mut i in 0..3 {
                                if (cy >= 0 && cy < TILE_SIZE as i32 * layer.ysize as i32) {
                                    let tile: uint16 = *layer.layout.wrapping_add(
                                        ((colX >> 4)
                                            + ((cy / TILE_SIZE as i32) << layer.widthShift))
                                            as usize,
                                    );
                                    if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                        let mask: int32 = collisionMasks[cPlane as usize]
                                            [(tile & 0xFFF) as usize]
                                            .floorMasks
                                            [(colX & 0xF) as usize]
                                            as i32;
                                        let ty: int32 = cy + mask;
                                        if (mask < 0xFF) {
                                            if (i32::abs(colY - ty) <= tolerance) {
                                                collided = true32;
                                                colY = ty;
                                            }
                                            i = 3;
                                        }
                                    }
                                }
                                cy += TILE_SIZE as i32;
                            }
                        }
                        posX = layer.position.x + colX;
                        posY = layer.position.y + colY;
                    }

                    layerID <<= 1;
                }

                if (collided == true32) {
                    entity.position.y = TO_FIXED!(posY) - yOffset;
                }
                return collided;
            }
            CollisionModes::CMODE_LWALL => {
                solid = if cPlane != 0 { (1 << 15) } else { (1 << 13) };

                for l in 0..LAYER_COUNT {
                    if (cLayers & layerID as u16) != 0 {
                        let layer = &tileLayers[l];
                        let mut colX: int32 = posX - layer.position.x;
                        let colY: int32 = posY - layer.position.y;
                        let mut cx: int32 = (colX & -(TILE_SIZE as i32)) - TILE_SIZE as i32;
                        if (colY >= 0 && colY < TILE_SIZE as i32 * layer.ysize as i32) {
                            for mut i in 0..3 {
                                if (cx >= 0 && cx < TILE_SIZE as i32 * layer.xsize as i32) {
                                    let tile: uint16 = *layer.layout.wrapping_add(
                                        ((cx >> 4)
                                            + ((colY / TILE_SIZE as i32) << layer.widthShift))
                                            as usize,
                                    );
                                    if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                        let mask: int32 = collisionMasks[cPlane as usize]
                                            [(tile & 0xFFF) as usize]
                                            .lWallMasks
                                            [(colY & 0xF) as usize]
                                            as i32;
                                        let tx: int32 = cx + mask;
                                        if (mask < 0xFF) {
                                            if (i32::abs(colX - tx) <= tolerance) {
                                                collided = true32;
                                                colX = tx;
                                            }
                                            i = 3;
                                        }
                                    }
                                }
                                cx += TILE_SIZE as i32;
                            }
                        }
                        posX = layer.position.x + colX;
                        posY = layer.position.y + colY;
                    }

                    layerID <<= 1;
                }

                if (collided == true32) {
                    entity.position.x = TO_FIXED!(posX) - xOffset;
                }
                return collided;
            }
            CollisionModes::CMODE_ROOF => {
                solid = if cPlane != 0 { (1 << 15) } else { (1 << 13) };

                for l in 0..LAYER_COUNT {
                    if (cLayers & layerID as u16) != 0 {
                        let layer = &tileLayers[l];
                        let colX: int32 = posX - layer.position.x;
                        let mut colY: int32 = posY - layer.position.y;
                        let mut cy: int32 = (colY & -(TILE_SIZE as i32)) + TILE_SIZE as i32;
                        if (colX >= 0 && colX < TILE_SIZE as i32 * layer.xsize as i32) {
                            for mut i in 0..3 {
                                if (cy >= 0 && cy < TILE_SIZE as i32 * layer.ysize as i32) {
                                    let tile: uint16 = *layer.layout.wrapping_add(
                                        ((colX >> 4)
                                            + ((cy / TILE_SIZE as i32) << layer.widthShift))
                                            as usize,
                                    );
                                    if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                        let mask: int32 = collisionMasks[cPlane as usize]
                                            [(tile & 0xFFF) as usize]
                                            .roofMasks
                                            [(colX & 0xF) as usize]
                                            as i32;
                                        let ty: int32 = cy + mask;
                                        if (mask < 0xFF) {
                                            if (i32::abs(colY - ty) <= tolerance) {
                                                collided = true32;
                                                colY = ty;
                                            }
                                            i = 3;
                                        }
                                    }
                                }
                                cy -= TILE_SIZE as i32;
                            }
                        }
                        posX = layer.position.x + colX;
                        posY = layer.position.y + colY;
                    }

                    layerID <<= 1;
                }

                if (collided == true32) {
                    entity.position.y = TO_FIXED!(posY) - yOffset;
                }
                return collided;
            }
            CollisionModes::CMODE_RWALL => {
                solid = if cPlane != 0 { (1 << 15) } else { (1 << 13) };

                for l in 0..LAYER_COUNT {
                    if (cLayers & layerID as u16) != 0 {
                        let layer = &tileLayers[l];
                        let mut colX: int32 = posX - layer.position.x;
                        let colY: int32 = posY - layer.position.y;
                        let mut cx: int32 = (colX & -(TILE_SIZE as i32)) + TILE_SIZE as i32;
                        if (colY >= 0 && colY < TILE_SIZE as i32 * layer.ysize as i32) {
                            for mut i in 0..3 {
                                if (cx >= 0 && cx < TILE_SIZE as i32 * layer.xsize as i32) {
                                    let tile: uint16 = *layer.layout.wrapping_add(
                                        ((cx >> 4)
                                            + ((colY / TILE_SIZE as i32) << layer.widthShift))
                                            as usize,
                                    );
                                    if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                        let mask: int32 = collisionMasks[cPlane as usize]
                                            [(tile & 0xFFF) as usize]
                                            .rWallMasks
                                            [(colY & 0xF) as usize]
                                            as i32;
                                        let tx: int32 = cx + mask;
                                        if (mask < 0xFF) {
                                            if (i32::abs(colX - tx) <= tolerance) {
                                                collided = true32;
                                                colX = tx;
                                            }
                                            i = 3;
                                        }
                                    }
                                }
                                cx -= TILE_SIZE as i32;
                            }
                        }
                        posX = layer.position.x + colX;
                        posY = layer.position.y + colY;
                    }

                    layerID <<= 1;
                }

                if (collided == true32) {
                    entity.position.x = TO_FIXED!(posX) - xOffset;
                }
                return collided;
            }
            _ => {
                return false32;
            }
        }
    }
}

#[no_mangle]
#[export_name = "RoofCollision"]
pub extern "C" fn roof_collision(sensor: &mut CollisionSensor) {
    let mut posX: int32 = FROM_FIXED!(sensor.position.x);
    let mut posY: int32 = FROM_FIXED!(sensor.position.y);

    let mut solid: int32 = 0;
    unsafe {
        if cfg!(feature = "version_u") {
            if ((*collisionEntity).tileCollisions == TileCollisionModes::TILECOLLISION_DOWN as i32)
            {
                solid = if (*collisionEntity).collisionPlane != 0 {
                    (1 << 15)
                } else {
                    (1 << 13)
                };
            } else {
                solid = if (*collisionEntity).collisionPlane != 0 {
                    (1 << 14)
                } else {
                    (1 << 12)
                };
            }
        } else {
            solid = if (*collisionEntity).collisionPlane != 0 {
                (1 << 15)
            } else {
                (1 << 13)
            };
        }

        let mut collideAngle: int32 = 0;
        let mut collidePos: int32 = -1;

        let mut layerID: i32 = 1;
        for l in 0..LAYER_COUNT {
            if ((*collisionEntity).collisionLayers & layerID as u8) != 0 {
                let layer = &tileLayers[l];
                let colX: int32 = posX - layer.position.x;
                let colY: int32 = posY - layer.position.y;
                let mut cy: int32 = (colY & -(TILE_SIZE as i32)) + TILE_SIZE as i32;

                if (colX >= 0 && colX < TILE_SIZE as i32 * layer.xsize as i32) {
                    let stepCount: int32 = if cfg!(feature = "version_u") { 2 } else { 3 };
                    for mut i in 0..stepCount {
                        let mut step: int32 = -(TILE_SIZE as i32);

                        if (cy >= 0 && cy < TILE_SIZE as i32 * layer.ysize as i32) {
                            let tileX: int32 = (colX / TILE_SIZE as i32);
                            let tileY: int32 = (cy / TILE_SIZE as i32);
                            let tile: uint16 = *layer
                                .layout
                                .wrapping_add((tileX + (tileY << layer.widthShift)) as usize);

                            if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                let mask: int32 = collisionMasks
                                    [(*collisionEntity).collisionPlane as usize]
                                    [tile as usize & 0xFFF]
                                    .roofMasks[colX as usize & 0xF]
                                    as i32;
                                let ty: int32 = if cfg!(feature = "version_u") {
                                    layer.position.y + cy + mask
                                } else {
                                    cy + mask
                                };
                                if (mask < 0xFF) {
                                    if cfg!(feature = "version_u") {
                                        step = TILE_SIZE as i32;
                                        if (colY > collidePos) {
                                            collideAngle = tileInfo
                                                [(*collisionEntity).collisionPlane as usize]
                                                [tile as usize & 0xFFF]
                                                .roofAngle
                                                as i32;
                                            collidePos = ty as i32;
                                            i = stepCount;
                                        }
                                    } else {
                                        if (colY < ty) {
                                            if (i32::abs(colY - ty) <= collisionMinimumDistance) {
                                                sensor.collided = true32;
                                                sensor.angle = tileInfo
                                                    [(*collisionEntity).collisionPlane as usize]
                                                    [tile as usize & 0xFFF]
                                                    .roofAngle
                                                    as u8;
                                                sensor.position.y =
                                                    TO_FIXED!(ty + layer.position.y);
                                                i = stepCount;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        cy += step;
                    }
                }

                posX = layer.position.x + colX;
                posY = layer.position.y + colY;
            }

            layerID <<= 1;
        }

        if cfg!(feature = "version_u") {
            if (collidePos >= 0
                && sensor.position.y <= TO_FIXED!(collidePos)
                && sensor.position.y - TO_FIXED!(collidePos) >= -collisionMinimumDistance)
            {
                sensor.angle = collideAngle as u8;
                sensor.position.y = TO_FIXED!(collidePos);
                sensor.collided = true32;
            }
        }
    }
}

#[no_mangle]
#[export_name = "RWallCollision"]
pub extern "C" fn r_wall_collision(sensor: &mut CollisionSensor) {
    let mut posX: int32 = FROM_FIXED!(sensor.position.x);
    let mut posY: int32 = FROM_FIXED!(sensor.position.y);

    unsafe {
        let solid: int32 = if (*collisionEntity).collisionPlane != 0 {
            1 << 15
        } else {
            1 << 13
        };

        let mut layerID = 1;
        for l in 0..LAYER_COUNT {
            if ((*collisionEntity).collisionLayers & layerID) != 0 {
                let layer = &tileLayers[l];
                let colX: int32 = posX - layer.position.x;
                let colY: int32 = posY - layer.position.y;
                let mut cx: int32 = (colX & -(TILE_SIZE as i32)) + TILE_SIZE as i32;

                if colY >= 0 && colY < TILE_SIZE as i32 * layer.ysize as i32 {
                    for mut i in 0..3 {
                        if cx >= 0 && cx < TILE_SIZE as i32 * layer.xsize as i32 {
                            let tile: uint16 = *layer.layout.wrapping_add(
                                (cx as usize / TILE_SIZE)
                                    + ((colY as usize / TILE_SIZE) << layer.widthShift),
                            );

                            if tile < 0xFFFF && (tile & solid as u16) != 0 {
                                let mask: int32 = collisionMasks
                                    [(*collisionEntity).collisionPlane as usize]
                                    [(tile & 0xFFF) as usize]
                                    .rWallMasks[(colY & 0xF) as usize]
                                    as i32;
                                let tx: int32 = cx + mask;
                                if mask < 0xFF && colX <= tx && i32::abs(colX - tx) <= 14 {
                                    sensor.collided = true32;
                                    sensor.angle = tileInfo
                                        [(*collisionEntity).collisionPlane as usize]
                                        [(tile & 0xFFF) as usize]
                                        .rWallAngle;
                                    sensor.position.x = TO_FIXED!(tx + layer.position.x);
                                    i = 3;
                                }
                            }
                        }

                        cx -= TILE_SIZE as i32;
                    }
                }

                posX = layer.position.x + colX;
                posY = layer.position.y + colY;

                layerID <<= 1;
            }
        }
    }
}
