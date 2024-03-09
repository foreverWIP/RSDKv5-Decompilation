use crate::*;

use self::{
    engine_core::math::{cos_256, sin_256},
    graphics::drawing::LAYER_COUNT,
};

use super::{collisionMasks, tileInfo, tileLayers, TILE_SIZE};

extern "C" {
    fn ProcessAirCollision_Down();
    #[cfg(feature = "version_u")]
    fn ProcessAirCollision_Up();
    fn SetPathGripSensors(cSensors: *mut CollisionSensor);
    fn FindFloorPosition(sensor: &CollisionSensor);
    fn FindRoofPosition(sensor: &CollisionSensor);
    fn FindLWallPosition(sensor: &CollisionSensor);
    fn FindRWallPosition(sensor: &CollisionSensor);
}

#[repr(i32)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TileCollisionModes {
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

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum CollisionModes {
    CMODE_FLOOR,
    CMODE_LWALL,
    CMODE_ROOF,
    CMODE_RWALL,
}

#[repr(C)]
pub struct Hitbox {
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
    unsafe {
        let solid_flags = match cMode {
            CollisionModes::CMODE_FLOOR => {
                if cPlane != 0 {
                    1 << 14
                } else {
                    1 << 12
                }
            }
            _ => {
                if cPlane != 0 {
                    1 << 15
                } else {
                    1 << 13
                }
            }
        };

        let mut collided: bool32 = false32;
        let mut posX: int32 = FROM_FIXED!(xOffset + entity.position.x);
        let mut posY: int32 = FROM_FIXED!(yOffset + entity.position.y);
        let mut layerID = 1;

        const TILE_SIZE: i32 = self::TILE_SIZE as i32;
        // check each tile layer to collide with (using a bitmask)
        for l in 0..LAYER_COUNT {
            // if we found a tile layer...
            if (cLayers & layerID) != 0 {
                let layer = &tileLayers[l];
                let mut colX = posX - layer.position.x;
                let mut colY = posY - layer.position.y;
                let colPrimaryAxis = match cMode {
                    CollisionModes::CMODE_FLOOR => colX,
                    CollisionModes::CMODE_LWALL => colY,
                    CollisionModes::CMODE_ROOF => colX,
                    CollisionModes::CMODE_RWALL => colY,
                };
                let colSecondaryAxis = match cMode {
                    CollisionModes::CMODE_FLOOR => &mut colY,
                    CollisionModes::CMODE_LWALL => &mut colX,
                    CollisionModes::CMODE_ROOF => &mut colY,
                    CollisionModes::CMODE_RWALL => &mut colX,
                };
                let layerSizePrimaryAxis = match cMode {
                    CollisionModes::CMODE_FLOOR => layer.xsize,
                    CollisionModes::CMODE_LWALL => layer.ysize,
                    CollisionModes::CMODE_ROOF => layer.xsize,
                    CollisionModes::CMODE_RWALL => layer.ysize,
                };
                let layerSizeSecondaryAxis = match cMode {
                    CollisionModes::CMODE_FLOOR => layer.ysize,
                    CollisionModes::CMODE_LWALL => layer.xsize,
                    CollisionModes::CMODE_ROOF => layer.ysize,
                    CollisionModes::CMODE_RWALL => layer.xsize,
                };
                let mut cAxis = (*colSecondaryAxis & -TILE_SIZE)
                    + match cMode {
                        CollisionModes::CMODE_FLOOR => -TILE_SIZE,
                        CollisionModes::CMODE_LWALL => TILE_SIZE,
                        CollisionModes::CMODE_ROOF => -TILE_SIZE,
                        CollisionModes::CMODE_RWALL => TILE_SIZE,
                    };
                if colPrimaryAxis >= 0 && colPrimaryAxis < TILE_SIZE * layerSizePrimaryAxis as i32 {
                    for i in 0..3 {
                        if cAxis >= 0 && cAxis < TILE_SIZE * layerSizeSecondaryAxis as i32 {
                            let tile = *layer.layout.wrapping_add(
                                ((colPrimaryAxis >> 4) + ((cAxis / TILE_SIZE) << layer.widthShift))
                                    as usize,
                            );
                            if tile < u16::MAX && (tile & solid_flags) != 0 {
                                let mask = match cMode {
                                    CollisionModes::CMODE_FLOOR => {
                                        collisionMasks[cPlane as usize][tile as usize & 0xFFF]
                                            .floorMasks
                                    }
                                    CollisionModes::CMODE_LWALL => {
                                        collisionMasks[cPlane as usize][tile as usize & 0xFFF]
                                            .lWallMasks
                                    }
                                    CollisionModes::CMODE_ROOF => {
                                        collisionMasks[cPlane as usize][tile as usize & 0xFFF]
                                            .roofMasks
                                    }
                                    CollisionModes::CMODE_RWALL => {
                                        collisionMasks[cPlane as usize][tile as usize & 0xFFF]
                                            .rWallMasks
                                    }
                                }[colPrimaryAxis as usize & 0xf]
                                    as i32;
                                let tAxis = cAxis + mask;
                                if mask < 0xff {
                                    if i32::abs(*colSecondaryAxis - tAxis) <= tolerance {
                                        collided = true32;
                                        *colSecondaryAxis = tAxis;
                                    }
                                    break;
                                }
                            }
                        }
                        cAxis += TILE_SIZE;
                    }
                }
                posX = layer.position.x + colX;
                posY = layer.position.y + colY;
            }

            layerID <<= 1;
        }

        if collided == true32 {
            match cMode {
                CollisionModes::CMODE_FLOOR => {
                    entity.position.y = TO_FIXED!(posY) - yOffset;
                }
                CollisionModes::CMODE_LWALL => {
                    entity.position.x = TO_FIXED!(posX) - xOffset;
                }
                CollisionModes::CMODE_ROOF => {
                    entity.position.y = TO_FIXED!(posY) - yOffset;
                }
                CollisionModes::CMODE_RWALL => {
                    entity.position.x = TO_FIXED!(posX) - xOffset;
                }
            }
        }

        collided
    }
}

#[no_mangle]
#[export_name = "ProcessObjectMovement"]
pub extern "C" fn process_object_movement(
    entity: &mut Entity,
    outerBox: &mut Hitbox,
    innerBox: &mut Hitbox,
) {
    if (!(entity as *mut Entity).is_null()
        && !(outerBox as *mut Hitbox).is_null()
        && !(innerBox as *mut Hitbox).is_null())
    {
        if (entity.tileCollisions as i32 != 0) {
            entity.angle &= 0xFF;

            unsafe {
                collisionTolerance = highCollisionTolerance as i32;
                if (entity.groundVel.abs() < TO_FIXED!(6) && entity.angle == 0) {
                    collisionTolerance = lowCollisionTolerance as i32;
                }

                collisionOuter.left = outerBox.left;
                collisionOuter.top = outerBox.top;
                collisionOuter.right = outerBox.right;
                collisionOuter.bottom = outerBox.bottom;

                collisionInner.left = innerBox.left;
                collisionInner.top = innerBox.top;
                collisionInner.right = innerBox.right;
                collisionInner.bottom = innerBox.bottom;

                collisionEntity = entity;

                #[cfg(feature = "version_u")]
                {
                    collisionMaskAir = if collisionOuter.bottom >= 14 { 19 } else { 17 };

                    if (entity.onGround == true32) {
                        // true = normal, false = flipped
                        if (entity.tileCollisions == TileCollisionModes::TILECOLLISION_DOWN) {
                            useCollisionOffset = (entity.angle == 0x00).into();
                        } else {
                            useCollisionOffset = (entity.angle == 0x80).into();
                        }

                        // fixes some clipping issues as chibi sonic (& using small hitboxes)
                        // shouldn't effect anything else :)
                        if (collisionOuter.bottom < 14) {
                            useCollisionOffset = false32;
                        }

                        process_path_grip();
                    } else {
                        useCollisionOffset = false32;
                        // true = normal, false = flipped
                        if (entity.tileCollisions == TileCollisionModes::TILECOLLISION_DOWN) {
                            ProcessAirCollision_Down();
                        } else {
                            ProcessAirCollision_Up();
                        }
                    }
                }
                #[cfg(not(feature = "version_u"))]
                {
                    if (collisionOuter.bottom >= 14) {
                        collisionOffset = COLLISION_OFFSET;
                        collisionMaskAir = 19;
                    } else {
                        collisionOffset = 0;
                        collisionTolerance = 15;
                        collisionMaskAir = 17;
                    }

                    if (entity.onGround) {
                        ProcessPathGrip();
                    } else {
                        ProcessAirCollision_Down();
                    }
                }
            }

            if (entity.onGround == true32) {
                entity.velocity.x = entity.groundVel * cos_256(entity.angle) >> 8;
                entity.velocity.y = entity.groundVel * sin_256(entity.angle) >> 8;
            } else {
                entity.groundVel = entity.velocity.x;
            }
        } else {
            entity.position.x += entity.velocity.x;
            entity.position.y += entity.velocity.y;
        }
    }
}

#[no_mangle]
#[export_name = "ProcessPathGrip"]
pub extern "C" fn process_path_grip() {
    let mut xVel: int32 = 0;
    let mut yVel: int32 = 0;

    unsafe {
        sensors[4].position.x = (*collisionEntity).position.x;
        sensors[4].position.y = (*collisionEntity).position.y;
        for i in 0..6 {
            sensors[i].angle = (*collisionEntity).angle as u8;
            sensors[i].collided = false32;
        }
        SetPathGripSensors(sensors.as_mut_ptr());

        let mut absSpeed: int32 = (*collisionEntity).groundVel.abs();
        let mut checkDist: int32 = absSpeed >> 18;
        absSpeed &= 0x3FFFF;
        while (checkDist > -1) {
            if (checkDist >= 1) {
                xVel = cos_256((*collisionEntity).angle) << 10;
                yVel = sin_256((*collisionEntity).angle) << 10;
                checkDist -= 1;
            } else {
                xVel = absSpeed * cos_256((*collisionEntity).angle) >> 8;
                yVel = absSpeed * sin_256((*collisionEntity).angle) >> 8;
                checkDist = -1;
            }

            if ((*collisionEntity).groundVel < 0) {
                xVel = -xVel;
                yVel = -yVel;
            }

            sensors[0].collided = false32;
            sensors[1].collided = false32;
            sensors[2].collided = false32;
            sensors[4].position.x += xVel;
            sensors[4].position.y += yVel;
            let mut tileDistance: int32 = -1;

            match ((*collisionEntity).collisionMode) {
                CollisionModes::CMODE_FLOOR => {
                    sensors[3].position.x += xVel;
                    sensors[3].position.y += yVel;

                    if ((*collisionEntity).groundVel > 0) {
                        l_wall_collision(&mut sensors[3]);
                        if cfg!(feature = "version_u") {
                            if (sensors[3].collided == true32) {
                                sensors[2].position.x = sensors[3].position.x - TO_FIXED!(2);
                            }
                        }
                    }

                    if ((*collisionEntity).groundVel < 0) {
                        r_wall_collision(&mut sensors[3]);
                        if cfg!(feature = "version_u") {
                            if (sensors[3].collided == true32) {
                                sensors[0].position.x = sensors[3].position.x + TO_FIXED!(2);
                            }
                        }
                    }

                    if (sensors[3].collided == true32) {
                        xVel = 0;
                        checkDist = -1;
                    }

                    for i in 0..3 {
                        sensors[i].position.x += xVel;
                        sensors[i].position.y += yVel;
                        FindFloorPosition(&sensors[i]);
                    }

                    tileDistance = -1;
                    for i in 0..3 {
                        if (tileDistance > -1) {
                            if (sensors[i].collided == true32) {
                                if (sensors[i].position.y
                                    < sensors[tileDistance as usize].position.y)
                                {
                                    tileDistance = i as i32;
                                }

                                if (sensors[i].position.y
                                    == sensors[tileDistance as usize].position.y
                                    && (sensors[i].angle < 0x08 || sensors[i].angle > 0xF8))
                                {
                                    tileDistance = i as i32;
                                }
                            }
                        } else if (sensors[i].collided == true32) {
                            tileDistance = i as i32;
                        }
                    }

                    if (tileDistance <= -1) {
                        checkDist = -1;
                    } else {
                        sensors[0].position.y = sensors[tileDistance as usize].position.y;
                        sensors[0].angle = sensors[tileDistance as usize].angle;

                        sensors[1].position.y = sensors[0].position.y;
                        sensors[1].angle = sensors[0].angle;

                        sensors[2].position.y = sensors[0].position.y;
                        sensors[2].angle = sensors[0].angle;

                        sensors[4].position.x = sensors[1].position.x;
                        sensors[4].position.y =
                            sensors[0].position.y - TO_FIXED!(collisionOuter.bottom);
                    }

                    if (sensors[0].angle < 0xDE && sensors[0].angle > 0x80) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_LWALL;
                    }
                    if (sensors[0].angle > 0x22 && sensors[0].angle < 0x80) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_RWALL;
                    }
                }

                CollisionModes::CMODE_LWALL => {
                    sensors[3].position.x += xVel;
                    sensors[3].position.y += yVel;

                    if ((*collisionEntity).groundVel > 0) {
                        roof_collision(&mut sensors[3]);
                    }

                    if ((*collisionEntity).groundVel < 0) {
                        floor_collision(&mut sensors[3]);
                    }

                    if (sensors[3].collided == true32) {
                        yVel = 0;
                        checkDist = -1;
                    }

                    for i in 0..3 {
                        sensors[i].position.x += xVel;
                        sensors[i].position.y += yVel;
                        FindLWallPosition(&sensors[i]);
                    }

                    tileDistance = -1;
                    for i in 0..3 {
                        if (tileDistance > -1) {
                            if (sensors[i].position.x < sensors[tileDistance as usize].position.x
                                && sensors[i].collided == true32)
                            {
                                tileDistance = i as i32;
                            }
                        } else if (sensors[i].collided == true32) {
                            tileDistance = i as i32;
                        }
                    }

                    if (tileDistance <= -1) {
                        checkDist = -1;
                    } else {
                        sensors[0].position.x = sensors[tileDistance as usize].position.x;
                        sensors[0].angle = sensors[tileDistance as usize].angle;

                        sensors[1].position.x = sensors[0].position.x;
                        sensors[1].angle = sensors[0].angle;

                        sensors[2].position.x = sensors[0].position.x;
                        sensors[2].angle = sensors[0].angle;

                        sensors[4].position.x =
                            sensors[1].position.x - TO_FIXED!(collisionOuter.bottom);
                        sensors[4].position.y = sensors[1].position.y;
                    }

                    if (sensors[0].angle > 0xE2) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_FLOOR;
                    }

                    if (sensors[0].angle < 0x9E) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_ROOF;
                    }
                }

                CollisionModes::CMODE_ROOF => {
                    sensors[3].position.x += xVel;
                    sensors[3].position.y += yVel;

                    if ((*collisionEntity).groundVel > 0) {
                        r_wall_collision(&mut sensors[3]);
                        if cfg!(feature = "version_u") {
                            if (sensors[3].collided == true32) {
                                sensors[2].position.x = sensors[3].position.x + TO_FIXED!(2);
                            }
                        }
                    }

                    if ((*collisionEntity).groundVel < 0) {
                        l_wall_collision(&mut sensors[3]);
                        if cfg!(feature = "version_u") {
                            if (sensors[3].collided == true32) {
                                sensors[0].position.x = sensors[3].position.x - TO_FIXED!(2);
                            }
                        }
                    }

                    if (sensors[3].collided == true32) {
                        xVel = 0;
                        checkDist = -1;
                    }

                    for i in 0..3 {
                        sensors[i].position.x += xVel;
                        sensors[i].position.y += yVel;
                        FindRoofPosition(&sensors[i]);
                    }

                    tileDistance = -1;
                    for i in 0..3 {
                        if (tileDistance > -1) {
                            if (sensors[i].position.y > sensors[tileDistance as usize].position.y
                                && sensors[i].collided == true32)
                            {
                                tileDistance = i as i32;
                            }
                        } else if (sensors[i].collided == true32) {
                            tileDistance = i as i32;
                        }
                    }

                    if (tileDistance <= -1) {
                        checkDist = -1;
                    } else {
                        sensors[0].position.y = sensors[tileDistance as usize].position.y;
                        sensors[0].angle = sensors[tileDistance as usize].angle;

                        sensors[1].position.y = sensors[0].position.y;
                        sensors[1].angle = sensors[0].angle;

                        sensors[2].position.y = sensors[0].position.y;
                        sensors[2].angle = sensors[0].angle;

                        sensors[4].position.x = sensors[1].position.x;
                        sensors[4].position.y =
                            sensors[0].position.y + TO_FIXED!(collisionOuter.bottom) + TO_FIXED!(1);
                    }

                    if (sensors[0].angle > 0xA2) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_LWALL;
                    }
                    if (sensors[0].angle < 0x5E) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_RWALL;
                    }
                }

                CollisionModes::CMODE_RWALL => {
                    sensors[3].position.x += xVel;
                    sensors[3].position.y += yVel;

                    if ((*collisionEntity).groundVel > 0) {
                        floor_collision(&mut sensors[3]);
                    }

                    if ((*collisionEntity).groundVel < 0) {
                        roof_collision(&mut sensors[3]);
                    }

                    if (sensors[3].collided == true32) {
                        yVel = 0;
                        checkDist = -1;
                    }

                    for i in 0..3 {
                        sensors[i].position.x += xVel;
                        sensors[i].position.y += yVel;
                        FindRWallPosition(&sensors[i]);
                    }

                    tileDistance = -1;
                    for i in 0..3 {
                        if (tileDistance > -1) {
                            if (sensors[i].position.x > sensors[tileDistance as usize].position.x
                                && sensors[i].collided == true32)
                            {
                                tileDistance = i as i32;
                            }
                        } else if (sensors[i].collided == true32) {
                            tileDistance = i as i32;
                        }
                    }

                    if (tileDistance <= -1) {
                        checkDist = -1;
                    } else {
                        sensors[0].position.x = sensors[tileDistance as usize].position.x;
                        sensors[0].angle = sensors[tileDistance as usize].angle;

                        sensors[1].position.x = sensors[0].position.x;
                        sensors[1].angle = sensors[0].angle;

                        sensors[2].position.x = sensors[0].position.x;
                        sensors[2].angle = sensors[0].angle;

                        sensors[4].position.x =
                            sensors[1].position.x + TO_FIXED!(collisionOuter.bottom) + TO_FIXED!(1);
                        sensors[4].position.y = sensors[1].position.y;
                    }

                    if (sensors[0].angle < 0x1E) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_FLOOR;
                    }
                    if (sensors[0].angle > 0x62) {
                        (*collisionEntity).collisionMode = CollisionModes::CMODE_ROOF;
                    }
                }
            }

            if (tileDistance != -1) {
                (*collisionEntity).angle = sensors[0].angle as i32;
            }

            if (sensors[3].collided == false32) {
                SetPathGripSensors(sensors.as_mut_ptr());
            } else {
                checkDist = -2;
            }
        }

        let newCollisionMode = if cfg!(feature = "version_u") {
            if (*collisionEntity).tileCollisions == TileCollisionModes::TILECOLLISION_DOWN {
                CollisionModes::CMODE_FLOOR
            } else {
                CollisionModes::CMODE_ROOF
            }
        } else {
            CollisionModes::CMODE_FLOOR
        };
        let newAngle: int32 = (newCollisionMode as i32) << 6;

        match ((*collisionEntity).collisionMode) {
            CollisionModes::CMODE_FLOOR => {
                if (sensors[0].collided == true32
                    || sensors[1].collided == true32
                    || sensors[2].collided == true32)
                {
                    (*collisionEntity).angle = sensors[0].angle as i32;

                    if (sensors[3].collided == false32) {
                        (*collisionEntity).position.x = sensors[4].position.x;
                    } else {
                        if ((*collisionEntity).groundVel > 0) {
                            (*collisionEntity).position.x =
                                sensors[3].position.x - TO_FIXED!(collisionOuter.right);
                        }

                        if ((*collisionEntity).groundVel < 0) {
                            (*collisionEntity).position.x = sensors[3].position.x
                                - TO_FIXED!(collisionOuter.left)
                                + TO_FIXED!(1);
                        }

                        (*collisionEntity).groundVel = 0;
                        (*collisionEntity).velocity.x = 0;
                    }

                    (*collisionEntity).position.y = sensors[4].position.y;
                } else {
                    (*collisionEntity).onGround = false32;
                    (*collisionEntity).collisionMode = newCollisionMode;
                    (*collisionEntity).velocity.x =
                        cos_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;
                    (*collisionEntity).velocity.y =
                        sin_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;
                    if ((*collisionEntity).velocity.y < -TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = -TO_FIXED!(16);
                    }

                    if ((*collisionEntity).velocity.y > TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = TO_FIXED!(16);
                    }

                    (*collisionEntity).groundVel = (*collisionEntity).velocity.x;
                    (*collisionEntity).angle = newAngle;
                    if (sensors[3].collided == false32) {
                        (*collisionEntity).position.x += (*collisionEntity).velocity.x;
                    } else {
                        if ((*collisionEntity).groundVel > 0) {
                            (*collisionEntity).position.x =
                                sensors[3].position.x - TO_FIXED!(collisionOuter.right);
                        }
                        if ((*collisionEntity).groundVel < 0) {
                            (*collisionEntity).position.x = sensors[3].position.x
                                - TO_FIXED!(collisionOuter.left)
                                + TO_FIXED!(1);
                        }

                        (*collisionEntity).groundVel = 0;
                        (*collisionEntity).velocity.x = 0;
                    }

                    (*collisionEntity).position.y += (*collisionEntity).velocity.y;
                }
            }

            CollisionModes::CMODE_LWALL => {
                if (sensors[0].collided == true32
                    || sensors[1].collided == true32
                    || sensors[2].collided == true32)
                {
                    (*collisionEntity).angle = sensors[0].angle as i32;
                } else {
                    (*collisionEntity).onGround = false32;
                    (*collisionEntity).collisionMode = newCollisionMode;
                    (*collisionEntity).velocity.x =
                        cos_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;
                    (*collisionEntity).velocity.y =
                        sin_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;

                    if ((*collisionEntity).velocity.y < -TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = -TO_FIXED!(16);
                    }

                    if ((*collisionEntity).velocity.y > TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = TO_FIXED!(16);
                    }

                    (*collisionEntity).groundVel = (*collisionEntity).velocity.x;
                    (*collisionEntity).angle = newAngle;
                }

                if (sensors[3].collided == false32) {
                    (*collisionEntity).position.x = sensors[4].position.x;
                    (*collisionEntity).position.y = sensors[4].position.y;
                } else {
                    if ((*collisionEntity).groundVel > 0) {
                        (*collisionEntity).position.y =
                            sensors[3].position.y + TO_FIXED!(collisionOuter.right) + TO_FIXED!(1);
                    }

                    if ((*collisionEntity).groundVel < 0) {
                        (*collisionEntity).position.y =
                            sensors[3].position.y - TO_FIXED!(collisionOuter.left);
                    }

                    (*collisionEntity).groundVel = 0;
                    (*collisionEntity).position.x = sensors[4].position.x;
                }
            }

            CollisionModes::CMODE_ROOF => {
                if (sensors[0].collided == true32
                    || sensors[1].collided == true32
                    || sensors[2].collided == true32)
                {
                    (*collisionEntity).angle = sensors[0].angle as i32;

                    if (sensors[3].collided == false32) {
                        (*collisionEntity).position.x = sensors[4].position.x;
                    } else {
                        if ((*collisionEntity).groundVel > 0) {
                            (*collisionEntity).position.x =
                                sensors[3].position.x + TO_FIXED!(collisionOuter.right);
                        }

                        if ((*collisionEntity).groundVel < 0) {
                            (*collisionEntity).position.x = sensors[3].position.x
                                + TO_FIXED!(collisionOuter.left)
                                - TO_FIXED!(1);
                        }

                        (*collisionEntity).groundVel = 0;
                    }
                } else {
                    (*collisionEntity).onGround = false32;
                    (*collisionEntity).collisionMode = newCollisionMode;
                    (*collisionEntity).velocity.x =
                        cos_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;
                    (*collisionEntity).velocity.y =
                        sin_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;

                    if ((*collisionEntity).velocity.y < -TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = -TO_FIXED!(16);
                    }

                    if ((*collisionEntity).velocity.y > TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = TO_FIXED!(16);
                    }

                    (*collisionEntity).angle = newAngle;
                    (*collisionEntity).groundVel = (*collisionEntity).velocity.x;

                    if (sensors[3].collided == false32) {
                        (*collisionEntity).position.x += (*collisionEntity).velocity.x;
                    } else {
                        if ((*collisionEntity).groundVel > 0) {
                            (*collisionEntity).position.x =
                                sensors[3].position.x - TO_FIXED!(collisionOuter.right);
                        }

                        if ((*collisionEntity).groundVel < 0) {
                            (*collisionEntity).position.x = sensors[3].position.x
                                - TO_FIXED!(collisionOuter.left)
                                + TO_FIXED!(1);
                        }

                        (*collisionEntity).groundVel = 0;
                    }
                }
                (*collisionEntity).position.y = sensors[4].position.y;
            }

            CollisionModes::CMODE_RWALL => {
                if (sensors[0].collided == true32
                    || sensors[1].collided == true32
                    || sensors[2].collided == true32)
                {
                    (*collisionEntity).angle = sensors[0].angle as i32;
                } else {
                    (*collisionEntity).onGround = false32;
                    (*collisionEntity).collisionMode = newCollisionMode;
                    (*collisionEntity).velocity.x =
                        cos_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;
                    (*collisionEntity).velocity.y =
                        sin_256((*collisionEntity).angle) * (*collisionEntity).groundVel >> 8;

                    if ((*collisionEntity).velocity.y < -TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = -TO_FIXED!(16);
                    }

                    if ((*collisionEntity).velocity.y > TO_FIXED!(16)) {
                        (*collisionEntity).velocity.y = TO_FIXED!(16);
                    }

                    (*collisionEntity).groundVel = (*collisionEntity).velocity.x;
                    (*collisionEntity).angle = newAngle;
                }

                if (sensors[3].collided == false32) {
                    (*collisionEntity).position.x = sensors[4].position.x;
                    (*collisionEntity).position.y = sensors[4].position.y;
                } else {
                    if ((*collisionEntity).groundVel > 0) {
                        (*collisionEntity).position.y =
                            sensors[3].position.y - TO_FIXED!(collisionOuter.right);
                    }

                    if ((*collisionEntity).groundVel < 0) {
                        (*collisionEntity).position.y =
                            sensors[3].position.y - TO_FIXED!(collisionOuter.left) + TO_FIXED!(1);
                    }

                    (*collisionEntity).groundVel = 0;
                    (*collisionEntity).position.x = sensors[4].position.x;
                }
            }

            _ => {}
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
            if ((*collisionEntity).tileCollisions == TileCollisionModes::TILECOLLISION_DOWN) {
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
#[export_name = "FloorCollision"]
pub extern "C" fn floor_collision(sensor: &mut CollisionSensor) {
    let mut posX: int32 = FROM_FIXED!(sensor.position.x);
    let mut posY: int32 = FROM_FIXED!(sensor.position.y);

    let mut solid: int32 = 0;
    unsafe {
        if cfg!(feature = "version_u") {
            if ((*collisionEntity).tileCollisions == TileCollisionModes::TILECOLLISION_DOWN) {
                solid = if (*collisionEntity).collisionPlane != 0 {
                    (1 << 14)
                } else {
                    (1 << 12)
                };
            } else {
                solid = if (*collisionEntity).collisionPlane != 0 {
                    (1 << 15)
                } else {
                    (1 << 13)
                };
            }
        } else {
            solid = if (*collisionEntity).collisionPlane != 0 {
                (1 << 14)
            } else {
                (1 << 12)
            };
        }

        let mut collideAngle: int32 = 0;
        let mut collidePos: int32 = 0x7FFFFFFF;

        let mut layerID = 1;
        for l in 0..LAYER_COUNT {
            if ((*collisionEntity).collisionLayers & layerID) != 0 {
                let layer = &tileLayers[l];
                let colX: int32 = posX - layer.position.x;
                let colY: int32 = posY - layer.position.y;
                let mut cy: int32 = (colY & -(TILE_SIZE as i32)) - TILE_SIZE as i32;

                if (colX >= 0 && colX < TILE_SIZE as i32 * layer.xsize as i32) {
                    let stepCount: i32 = if cfg!(feature = "version_u") { 2 } else { 3 };
                    for mut i in 0..stepCount {
                        let mut step: int32 = TILE_SIZE as i32;

                        if (cy >= 0 && cy < TILE_SIZE as i32 * layer.ysize as i32) {
                            let tile: uint16 = *layer.layout.wrapping_add(
                                (colX as usize / TILE_SIZE)
                                    + ((cy as usize / TILE_SIZE) << layer.widthShift),
                            );
                            if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                let mask: int32 = collisionMasks
                                    [(*collisionEntity).collisionPlane as usize]
                                    [tile as usize & 0xFFF]
                                    .floorMasks[colX as usize & 0xF]
                                    as i32;
                                let ty: i32 = if cfg!(feature = "version_u") {
                                    layer.position.y + cy + mask
                                } else {
                                    cy + mask
                                };
                                if (mask < 0xFF) {
                                    if cfg!(feature = "version_u") {
                                        step = -(TILE_SIZE as i32);
                                        if (colY < collidePos) {
                                            collideAngle = tileInfo
                                                [(*collisionEntity).collisionPlane as usize]
                                                [tile as usize & 0xFFF]
                                                .floorAngle
                                                as i32;
                                            collidePos = ty;
                                            i = stepCount;
                                        }
                                    } else {
                                        if (colY >= ty) {
                                            if (i32::abs(colY - ty) <= collisionMinimumDistance) {
                                                sensor.collided = true32;
                                                sensor.angle = tileInfo
                                                    [(*collisionEntity).collisionPlane as usize]
                                                    [tile as usize & 0xFFF]
                                                    .floorAngle;
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
            if (collidePos != 0x7FFFFFFF) {
                let collideDist: int32 = sensor.position.y - TO_FIXED!(collidePos);
                if (sensor.position.y >= TO_FIXED!(collidePos)
                    && collideDist <= collisionMinimumDistance)
                {
                    sensor.angle = collideAngle as u8;
                    sensor.position.y = TO_FIXED!(collidePos);
                    sensor.collided = true32;
                }
            }
        }
    }
}

#[no_mangle]
#[export_name = "LWallCollision"]
pub extern "C" fn l_wall_collision(sensor: &mut CollisionSensor) {
    let mut posX: int32 = FROM_FIXED!(sensor.position.x);
    let mut posY: int32 = FROM_FIXED!(sensor.position.y);

    unsafe {
        let solid: int32 = if (*collisionEntity).collisionPlane != 0 {
            (1 << 15)
        } else {
            (1 << 13)
        };

        let mut layerID = 1;
        for l in 0..LAYER_COUNT {
            if ((*collisionEntity).collisionLayers & layerID) != 0 {
                let layer = &tileLayers[l];
                let colX: int32 = posX - layer.position.x;
                let colY: int32 = posY - layer.position.y;
                let mut cx: int32 = (colX & -(TILE_SIZE as i32)) - TILE_SIZE as i32;

                if (colY >= 0 && colY < TILE_SIZE as i32 * layer.ysize as i32) {
                    for mut i in 0..3 {
                        if (cx >= 0 && cx < TILE_SIZE as i32 * layer.xsize as i32) {
                            let tile: uint16 = *layer.layout.wrapping_add(
                                (cx as usize / TILE_SIZE)
                                    + ((colY as usize / TILE_SIZE) << layer.widthShift),
                            );

                            if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                let mask: int32 = collisionMasks
                                    [(*collisionEntity).collisionPlane as usize]
                                    [tile as usize & 0xFFF]
                                    .lWallMasks[colY as usize & 0xF]
                                    as i32;
                                let tx: int32 = cx + mask;
                                if (mask < 0xFF && colX >= tx && i32::abs(colX - tx) <= 14) {
                                    sensor.collided = true32;
                                    sensor.angle = tileInfo
                                        [(*collisionEntity).collisionPlane as usize]
                                        [tile as usize & 0xFFF]
                                        .lWallAngle;
                                    sensor.position.x = TO_FIXED!(tx + layer.position.x);
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
    }
}

#[no_mangle]
#[export_name = "RWallCollision"]
pub extern "C" fn r_wall_collision(sensor: &mut CollisionSensor) {
    let mut posX: int32 = FROM_FIXED!(sensor.position.x);
    let mut posY: int32 = FROM_FIXED!(sensor.position.y);

    unsafe {
        let solid: int32 = if (*collisionEntity).collisionPlane != 0 {
            (1 << 15)
        } else {
            (1 << 13)
        };

        let mut layerID = 1;
        for l in 0..LAYER_COUNT {
            if ((*collisionEntity).collisionLayers & layerID) != 0 {
                let layer = &tileLayers[l];
                let colX: int32 = posX - layer.position.x;
                let colY: int32 = posY - layer.position.y;
                let mut cx: int32 = (colX & -(TILE_SIZE as i32)) + TILE_SIZE as i32;

                if (colY >= 0 && colY < TILE_SIZE as i32 * layer.ysize as i32) {
                    for mut i in 0..3 {
                        if (cx >= 0 && cx < TILE_SIZE as i32 * layer.xsize as i32) {
                            let tile: uint16 = *layer.layout.wrapping_add(
                                (cx as usize / TILE_SIZE)
                                    + ((colY as usize / TILE_SIZE) << layer.widthShift),
                            );

                            if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                let mask: int32 = collisionMasks
                                    [(*collisionEntity).collisionPlane as usize]
                                    [tile as usize & 0xFFF]
                                    .rWallMasks[colY as usize & 0xF]
                                    as i32;
                                let tx: int32 = cx + mask;
                                if (mask < 0xFF && colX <= tx && i32::abs(colX - tx) <= 14) {
                                    sensor.collided = true32;
                                    sensor.angle = tileInfo
                                        [(*collisionEntity).collisionPlane as usize]
                                        [tile as usize & 0xFFF]
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
            }

            layerID <<= 1;
        }
    }
}

#[no_mangle]
#[export_name = "FindRWallPosition"]
pub extern "C" fn find_r_wall_position(sensor: &mut CollisionSensor) {
    let mut posX: int32 = FROM_FIXED!(sensor.position.x);
    let mut posY: int32 = FROM_FIXED!(sensor.position.y);

    unsafe {
        let solid: int32 = if (*collisionEntity).collisionPlane != 0 {
            ((1 << 14) | (1 << 15))
        } else {
            ((1 << 12) | (1 << 13))
        };

        let mut startX: int32 = posX;

        let mut layerID = 1;
        for l in 0..LAYER_COUNT {
            if ((*collisionEntity).collisionLayers & layerID) != 0 {
                let layer = &tileLayers[l];
                let colX: int32 = posX - layer.position.x;
                let colY: int32 = posY - layer.position.y;
                let mut cx: int32 = (colX & -(TILE_SIZE as i32)) + TILE_SIZE as i32;

                if (colY >= 0 && colY < TILE_SIZE as i32 * layer.ysize as i32) {
                    for mut i in 0..3 {
                        if (cx >= 0 && cx < TILE_SIZE as i32 * layer.xsize as i32) {
                            let tile: uint16 = *layer.layout.wrapping_add(
                                (cx as usize / TILE_SIZE)
                                    + ((colY as usize / TILE_SIZE) << layer.widthShift),
                            );

                            if (tile < 0xFFFF && (tile & solid as u16) != 0) {
                                let mask: int32 = collisionMasks
                                    [(*collisionEntity).collisionPlane as usize]
                                    [tile as usize & 0xFFF]
                                    .rWallMasks[colY as usize & 0xF]
                                    as i32;
                                let tx: int32 = cx + mask;
                                let tileAngle: int32 =
                                    tileInfo[(*collisionEntity).collisionPlane as usize]
                                        [tile as usize & 0xFFF]
                                        .rWallAngle as i32;

                                if (mask < 0xFF) {
                                    if (sensor.collided == false32 || startX <= tx) {
                                        if (i32::abs(colX - tx) <= collisionTolerance
                                            && i32::abs(sensor.angle as i32 - tileAngle)
                                                <= wallAngleTolerance as i32)
                                        {
                                            sensor.collided = true32;
                                            sensor.angle = tileAngle as u8;
                                            sensor.position.x = TO_FIXED!(tx + layer.position.x);
                                            startX = tx;
                                            i = 3;
                                        }
                                    }
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
    }
}
