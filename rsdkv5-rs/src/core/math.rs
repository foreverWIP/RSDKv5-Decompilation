extern "C" {
    static sin256LookupTable: [i32; 0x100];
    static cos256LookupTable: [i32; 0x100];
    static tan256LookupTable: [i32; 0x100];
    static asin256LookupTable: [i32; 0x100];
    static acos256LookupTable: [i32; 0x100];

    static arcTan256LookupTable: [u8; 0x100 * 0x100];
}

#[no_mangle]
#[export_name = "RSDK_Sin256"]
pub extern "C" fn sin_256(angle: i32) -> i32 {
    unsafe { sin256LookupTable[(angle & 0xFF) as usize] }
}

#[no_mangle]
#[export_name = "RSDK_Cos256"]
pub extern "C" fn cos_256(angle: i32) -> i32 {
    unsafe { cos256LookupTable[(angle & 0xFF) as usize] }
}

#[no_mangle]
#[export_name = "RSDK_Tan256"]
pub extern "C" fn tan_256(angle: i32) -> i32 {
    unsafe { tan256LookupTable[(angle & 0xFF) as usize] }
}

#[no_mangle]
#[export_name = "RSDK_ASin256"]
pub extern "C" fn asin_256(angle: i32) -> i32 {
    unsafe {
        if angle > 0xFF {
            0
        } else {
            if angle < 0 {
                -asin256LookupTable[(-angle) as usize]
            } else {
                asin256LookupTable[angle as usize]
            }
        }
    }
}

#[no_mangle]
#[export_name = "RSDK_ACos256"]
pub extern "C" fn acos_256(angle: i32) -> i32 {
    unsafe {
        if angle > 0xFF {
            0
        } else {
            if angle < 0 {
                -acos256LookupTable[(-angle) as usize]
            } else {
                acos256LookupTable[angle as usize]
            }
        }
    }
}
