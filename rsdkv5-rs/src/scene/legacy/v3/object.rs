use crate::*;

#[repr(C)]
pub struct Entity {
    pub XPos: int32,
    pub YPos: int32,
    pub values: [int32; 8],
    pub scale: int32,
    pub rotation: int32,
    pub animationTimer: int32,
    pub animationSpeed: int32,
    pub type_: uint8,
    pub propertyValue: uint8,
    pub state: uint8,
    pub priority: uint8,
    pub drawOrder: uint8,
    pub direction: uint8,
    pub inkEffect: uint8,
    pub alpha: uint8,
    pub animation: uint8,
    pub prevAnimation: uint8,
    pub frame: uint8,
}
