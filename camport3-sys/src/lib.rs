#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub use TY_STATUS_LIST::*;
pub use TY_INTERFACE_TYPE_LIST::*;
pub use TY_FW_ERRORCODE_LIST::*;

// pub fn num2enum<A: NoUninit, B: AnyBitPattern>(a: A) -> B {
//     bytemuck::try_cast(a).unwrap()
// }

// unsafe impl Zeroable for TY_FW_ERRORCODE_LIST {}
// unsafe impl Pod for TY_FW_ERRORCODE_LIST {}

