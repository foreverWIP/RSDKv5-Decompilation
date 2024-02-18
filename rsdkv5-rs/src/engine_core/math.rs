use std::time::{SystemTime, UNIX_EPOCH};

use crate::*;

// #define SET_BIT(value, set, pos) ((value) ^= (-(int32)(set) ^ (value)) & (1 << (pos)))
// #define GET_BIT(b, pos)          ((b) >> (pos)&1)

// #define TO_FIXED(x)   ((x) << 16)
#[macro_export]
macro_rules! TO_FIXED {
    ($x:expr) => {
        $x << 16
    };
}
// #define FROM_FIXED(x) ((x) >> 16)

// floating point variants
// #define TO_FIXED_F(x)   ((x)*65536.0)
// #define FROM_FIXED_F(x) ((x) / 65536.0)

#[repr(C)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}

#[no_mangle]
pub static mut sin256LookupTable: [i32; 0x100] = [0i32; 0x100];
#[no_mangle]
pub static mut cos256LookupTable: [i32; 0x100] = [0i32; 0x100];
#[no_mangle]
pub static mut tan256LookupTable: [i32; 0x100] = [0i32; 0x100];
#[no_mangle]
pub static mut asin256LookupTable: [i32; 0x100] = [0i32; 0x100];
#[no_mangle]
pub static mut acos256LookupTable: [i32; 0x100] = [0i32; 0x100];
#[no_mangle]
pub static mut sin512LookupTable: [i32; 0x200] = [0i32; 0x200];
#[no_mangle]
pub static mut cos512LookupTable: [i32; 0x200] = [0i32; 0x200];
#[no_mangle]
pub static mut tan512LookupTable: [i32; 0x200] = [0i32; 0x200];
#[no_mangle]
pub static mut asin512LookupTable: [i32; 0x200] = [0i32; 0x200];
#[no_mangle]
pub static mut acos512LookupTable: [i32; 0x200] = [0i32; 0x200];
#[no_mangle]
pub static mut sin1024LookupTable: [i32; 0x400] = [0i32; 0x400];
#[no_mangle]
pub static mut cos1024LookupTable: [i32; 0x400] = [0i32; 0x400];
#[no_mangle]
pub static mut tan1024LookupTable: [i32; 0x400] = [0i32; 0x400];
#[no_mangle]
pub static mut asin1024LookupTable: [i32; 0x400] = [0i32; 0x400];
#[no_mangle]
pub static mut acos1024LookupTable: [i32; 0x400] = [0i32; 0x400];

#[no_mangle]
pub static mut randSeed: u32 = 0;

#[no_mangle]
pub static mut arcTan256LookupTable: [u8; 0x100 * 0x100] = [0u8; 0x100 * 0x100];

#[no_mangle]
#[export_name = "RSDK_CalculateTrigAngles"]
pub extern "C" fn calc_trig_angles() {
    // M_PI is *too* accurate, so use this instead
    const RSDK_PI: f32 = 3.1415927;

    unsafe {
        randSeed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        for i in 0..0x400 {
            let i_f = i as f32;
            sin1024LookupTable[i] = (f32::sin((i_f / 512.0) * RSDK_PI) * 1024.0) as int32;
            cos1024LookupTable[i] = (f32::cos((i_f / 512.0) * RSDK_PI) * 1024.0) as int32;
            tan1024LookupTable[i] = (f32::tan((i_f / 512.0) * RSDK_PI) * 1024.0) as int32;
            asin1024LookupTable[i] = ((f32::asin(i_f / 1023.0) * 512.0) / RSDK_PI) as int32;
            acos1024LookupTable[i] = ((f32::acos(i_f / 1023.0) * 512.0) / RSDK_PI) as int32;
        }

        cos1024LookupTable[0x000] = 0x400;
        cos1024LookupTable[0x100] = 0;
        cos1024LookupTable[0x200] = -0x400;
        cos1024LookupTable[0x300] = 0;

        sin1024LookupTable[0x000] = 0;
        sin1024LookupTable[0x100] = 0x400;
        sin1024LookupTable[0x200] = 0;
        sin1024LookupTable[0x300] = -0x400;

        for i in 0..0x200 {
            let i_f = i as f32;
            sin512LookupTable[i] = (f32::sin((i_f / 256.0) * RSDK_PI) * 512.0) as int32;
            cos512LookupTable[i] = (f32::cos((i_f / 256.0) * RSDK_PI) * 512.0) as int32;
            tan512LookupTable[i] = (f32::tan((i_f / 256.0) * RSDK_PI) * 512.0) as int32;
            asin512LookupTable[i] = ((f32::asin(i_f / 511.0) * 256.0) / RSDK_PI) as int32;
            acos512LookupTable[i] = ((f32::acos(i_f / 511.0) * 256.0) / RSDK_PI) as int32;
        }

        cos512LookupTable[0x00] = 0x200;
        cos512LookupTable[0x80] = 0;
        cos512LookupTable[0x100] = -0x200;
        cos512LookupTable[0x180] = 0;

        sin512LookupTable[0x00] = 0;
        sin512LookupTable[0x80] = 0x200;
        sin512LookupTable[0x100] = 0;
        sin512LookupTable[0x180] = -0x200;

        for i in 0..0x100 {
            let i_f = i as f32;
            sin256LookupTable[i] = (sin512LookupTable[i * 2] >> 1) as int32;
            cos256LookupTable[i] = (cos512LookupTable[i * 2] >> 1) as int32;
            tan256LookupTable[i] = (tan512LookupTable[i * 2] >> 1) as int32;
            asin256LookupTable[i] = ((f32::asin(i_f / 255.0) * 128.0) / RSDK_PI) as int32;
            acos256LookupTable[i] = ((f32::acos(i_f / 255.0) * 128.0) / RSDK_PI) as int32;
        }

        for y in 0..0x100 {
            for x in 0..0x100 {
                // 40.743664 = 0x100 / (2 * M_PI) (roughly)
                arcTan256LookupTable[x * 0x100 + y] =
                    ((f32::atan2(y as f32, x as f32) * 40.743664) as i32) as u8;
            }
        }
    }
}

const fn get_trig<const TABLE_SIZE: usize>(
    angle: i32,
    table_256: &[i32; 0x100],
    table_512: &[i32; 0x200],
    table_1024: &[i32; 0x400],
) -> i32 {
    match TABLE_SIZE {
        0x100 => table_256[(angle as usize) & 0xff],
        0x200 => table_512[(angle as usize) & 0x1ff],
        0x400 => table_1024[(angle as usize) & 0x3ff],
        _ => unreachable!(),
    }
}

fn get_sin<const TABLE_SIZE: usize>(angle: i32) -> i32 {
    unsafe {
        get_trig::<TABLE_SIZE>(
            angle,
            &sin256LookupTable,
            &sin512LookupTable,
            &sin1024LookupTable,
        )
    }
}

fn get_cos<const TABLE_SIZE: usize>(angle: i32) -> i32 {
    unsafe {
        get_trig::<TABLE_SIZE>(
            angle,
            &cos256LookupTable,
            &cos512LookupTable,
            &cos1024LookupTable,
        )
    }
}

fn get_tan<const TABLE_SIZE: usize>(angle: i32) -> i32 {
    unsafe {
        get_trig::<TABLE_SIZE>(
            angle,
            &tan256LookupTable,
            &tan512LookupTable,
            &tan1024LookupTable,
        )
    }
}

fn get_atrig<const TABLE_SIZE: usize>(angle: i32, table: &[i32; TABLE_SIZE]) -> i32 {
    if angle > (TABLE_SIZE as i32 - 1) {
        0
    } else {
        if angle < 0 {
            -table[(-angle) as usize]
        } else {
            table[angle as usize]
        }
    }
}

#[no_mangle]
#[export_name = "RSDK_Sin256"]
pub extern "C" fn sin_256(angle: i32) -> i32 {
    get_sin::<0x100>(angle)
}

#[no_mangle]
#[export_name = "RSDK_Cos256"]
pub extern "C" fn cos_256(angle: i32) -> i32 {
    get_cos::<0x100>(angle)
}

#[no_mangle]
#[export_name = "RSDK_Tan256"]
pub extern "C" fn tan_256(angle: i32) -> i32 {
    get_tan::<0x100>(angle)
}

#[no_mangle]
#[export_name = "RSDK_ASin256"]
pub extern "C" fn asin_256(angle: i32) -> i32 {
    unsafe { get_atrig(angle, &asin256LookupTable) }
}

#[no_mangle]
#[export_name = "RSDK_ACos256"]
pub extern "C" fn acos_256(angle: i32) -> i32 {
    unsafe { get_atrig(angle, &acos256LookupTable) }
}

#[no_mangle]
#[export_name = "RSDK_Sin512"]
pub extern "C" fn sin_512(angle: i32) -> i32 {
    get_sin::<0x200>(angle)
}

#[no_mangle]
#[export_name = "RSDK_Cos512"]
pub extern "C" fn cos_512(angle: i32) -> i32 {
    get_cos::<0x200>(angle)
}

#[no_mangle]
#[export_name = "RSDK_Tan512"]
pub extern "C" fn tan_512(angle: i32) -> i32 {
    get_tan::<0x200>(angle)
}

#[no_mangle]
#[export_name = "RSDK_ASin512"]
pub extern "C" fn asin_512(angle: i32) -> i32 {
    unsafe { get_atrig(angle, &asin512LookupTable) }
}

#[no_mangle]
#[export_name = "RSDK_ACos512"]
pub extern "C" fn acos_512(angle: i32) -> i32 {
    unsafe { get_atrig(angle, &acos512LookupTable) }
}

#[no_mangle]
#[export_name = "RSDK_Sin1024"]
pub extern "C" fn sin_1024(angle: i32) -> i32 {
    get_sin::<0x400>(angle)
}

#[no_mangle]
#[export_name = "RSDK_Cos1024"]
pub extern "C" fn cos_1024(angle: i32) -> i32 {
    get_cos::<0x400>(angle)
}

#[no_mangle]
#[export_name = "RSDK_Tan1024"]
pub extern "C" fn tan_1024(angle: i32) -> i32 {
    get_tan::<0x400>(angle)
}

#[no_mangle]
#[export_name = "RSDK_ASin1024"]
pub extern "C" fn asin_1024(angle: i32) -> i32 {
    unsafe { get_atrig(angle, &asin1024LookupTable) }
}

#[no_mangle]
#[export_name = "RSDK_ACos1024"]
pub extern "C" fn acos_1024(angle: i32) -> i32 {
    unsafe { get_atrig(angle, &acos1024LookupTable) }
}

#[no_mangle]
#[export_name = "RSDK_ArcTanLookup"]
pub extern "C" fn atan2_lookup(x: i32, y: i32) -> u8 {
    let mut x_abs: i32 = x.abs();
    let mut y_abs: i32 = y.abs();

    if x_abs <= y_abs {
        while y_abs > 0xFF {
            x_abs >>= 4;
            y_abs >>= 4;
        }
    } else {
        while x_abs > 0xFF {
            x_abs >>= 4;
            y_abs >>= 4;
        }
    }
    unsafe {
        if x <= 0 {
            if y <= 0 {
                arcTan256LookupTable[((x_abs << 8) + y_abs) as usize] + 0x80
            } else {
                0x80 - arcTan256LookupTable[((x_abs << 8) + y_abs) as usize]
            }
        } else if y <= 0 {
            (-(arcTan256LookupTable[((x_abs << 8) + y_abs) as usize] as i8)) as u8
        } else {
            arcTan256LookupTable[((x_abs << 8) + y_abs) as usize]
        }
    }
}

#[no_mangle]
#[export_name = "RSDK_SetRandSeed"]
pub extern "C" fn set_rand_seed(key: i32) {
    unsafe {
        randSeed = key as u32;
    }
}

#[no_mangle]
#[export_name = "RSDK_Rand"]
pub extern "C" fn rand(min: i32, max: i32) -> i32 {
    unsafe {
        let seed1: u32 = randSeed * 0x41c64e6d + 0x3039;
        let seed2: u32 = seed1 * 0x41c64e6d + 0x3039;
        randSeed = seed2 * 0x41c64e6d + 0x3039;
        let res: i32 = (((seed1 >> 0x10 & 0x7ff) << 10 ^ seed2 >> 0x10 & 0x7ff) << 10
            ^ randSeed >> 0x10 & 0x7ff) as i32;
        if min < max {
            min + res % (max - min)
        } else {
            max + res % (min - max)
        }
    }
}

#[no_mangle]
#[export_name = "RSDK_RandSeeded"]
pub extern "C" fn rand_seeded(min: i32, max: i32, rand_seed: *mut i32) -> i32 {
    if rand_seed.is_null() {
        return 0;
    }

    unsafe {
        let seed1: u32 = (*rand_seed as u32) * 0x41c64e6d + 0x3039;
        let seed2: u32 = seed1 * 0x41c64e6d + 0x3039;
        *rand_seed = (seed2 * 0x41c64e6d + 0x3039) as i32;
        let res: i32 = (((seed1 >> 0x10 & 0x7ff) << 10 ^ seed2 >> 0x10 & 0x7ff) << 10
            ^ (*rand_seed >> 0x10 & 0x7ff) as u32) as i32;
        if min < max {
            min + res % (max - min)
        } else {
            max + res % (min - max)
        }
    }
}
