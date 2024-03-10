use crate::*;

#[repr(i32)]
pub enum PrintModes {
    PRINT_NORMAL,
    PRINT_POPUP,
    PRINT_ERROR,
    PRINT_FATAL,
    #[cfg(feature = "version_u")]
    PRINT_SCRIPTERR,
}

extern "C" {
    pub fn PrintLog(mode: PrintModes, message: *const i8, ...);
}
