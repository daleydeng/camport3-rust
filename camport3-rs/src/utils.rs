use std::ffi::CStr;
use std::ops::BitAnd;

pub(crate) fn cstr_to_str<'a>(s: *const i8) -> &'a str {
    unsafe {CStr::from_ptr(s).to_str().unwrap()}
}

pub(crate) fn bit_is_set<T>(a: T, b: T) -> bool
where T: BitAnd + Copy,
    <T as BitAnd>::Output: PartialEq<T>,
{
    (a & b).eq(&b)
}