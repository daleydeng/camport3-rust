use std::{ffi::CStr, mem::transmute};

pub fn cstr_to_str<'a>(s: *const i8) -> &'a str {
    unsafe {CStr::from_ptr(s).to_str().unwrap()}
}