use std::{ffi::OsStr};
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

pub fn to_wide(msg:&str) -> Vec<u16> {
    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    wide
}
