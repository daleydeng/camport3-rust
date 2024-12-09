use std::{fmt::{write, Display}, mem::transmute, net::IpAddr, ops::Deref, str::FromStr};

use macaddr::{MacAddr, MacAddr6};
use thiserror::Error;
use strum_macros::FromRepr;
use bitflags::bitflags;
use camport3_sys::*;

mod utils;
use utils::*;

#[derive(FromRepr)]
pub enum TestEnum {
    OK = 0,
}

#[derive(Error, Debug, PartialEq, Eq, FromRepr)]
pub enum DeviceError {
    #[error("error")]
    ERROR = 1001,
    #[error("not inited")]
    NotInited = 1002,
    // TODO
    // NOT_IMPLEMENTED = -1003,
    // NOT_PERMITTED = -1004,
    // DEVICE_ERROR = -1005,
    // INVALID_PARAMETER = -1006,
    // INVALID_HANDLE = -1007,
    // INVALID_COMPONENT = -1008,
    // INVALID_FEATURE = -1009,
    // WRONG_TYPE = -1010,
    // WRONG_SIZE = -1011,
    // OUT_OF_MEMORY = -1012,
    // OUT_OF_RANGE = -1013,
    // TIMEOUT = -1014,
    // WRONG_MODE = -1015,
    // BUSY = -1016,
    // IDLE = -1017,
    // NO_DATA = -1018,
    // NO_BUFFER = -1019,
    // NULL_POINTER = -1020,
    // READONLY_FEATURE = -1021,
    // INVALID_DESCRIPTOR = -1022,
    // INVALID_INTERFACE = -1023,
    // FIRMWARE_ERROR = -1024,
    // DEV_EPERM = -1,
    // DEV_EIO = -5,
    // DEV_ENOMEM = -12,
    // DEV_EBUSY = -16,
    // DEV_EINVAL = -22,
}

impl From<i32> for DeviceError {
    fn from(value: i32) -> Self {
        DeviceError::from_repr(-value as usize).unwrap()
    }
}

impl From<u32> for DeviceError {
    fn from(value: u32) -> Self {
        DeviceError::from_repr(value as usize).unwrap()
    }
}

type Result<T> = std::result::Result<T, DeviceError>;

fn chkerr(status: TY_STATUS) -> Result<()> {
    if status != TY_STATUS_LIST::TY_STATUS_OK {
        Err(status.into())
    } else {
        Ok(())
    }
}

pub fn ty_error_string(status: TY_STATUS) -> &'static str {
    cstr_to_str(unsafe{TYErrorString(status)})
}

#[repr(transparent)]
pub struct VersionInfo(TY_VERSION_INFO);

impl VersionInfo {
    pub fn major(&self) -> i32 {
        self.0.major
    }
    pub fn minor(&self) -> i32 {
        self.0.minor
    }
    pub fn patch(&self) -> i32 {
        self.0.patch
    }
}
impl From<VersionInfo> for (i32, i32, i32) {
    fn from(value: VersionInfo) -> Self {
        (value.major(), value.minor(), value.patch())
    }
}

impl AsRef<VersionInfo> for TY_VERSION_INFO {
    fn as_ref(&self) -> &VersionInfo {
        unsafe { transmute(self) }
    }
}

impl Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}

#[repr(transparent)]
pub struct InterfaceInfo(TY_INTERFACE_INFO);

impl InterfaceInfo {
    pub fn name(&self) -> &str {
        cstr_to_str(self.0.name.as_ptr())
    }

    pub fn id(&self) -> &str {
        cstr_to_str(self.0.id.as_ptr())
    }

    pub fn type_(&self) -> InterfaceType {
       InterfaceType::from_bits(self.0.type_).unwrap()
    }

    pub fn net_info(&self) -> Option<&NetInfo> {
        if self.type_().contains(InterfaceType::USB) {
            None
        } else {
            Some(unsafe { transmute(&self.0.netInfo) }) // better way?
        }
    }
}

impl AsRef<InterfaceInfo> for TY_INTERFACE_INFO {
    fn as_ref(&self) -> &InterfaceInfo {
        unsafe { transmute(self) }
    }
}

bitflags! {
    // Attributes can be applied to flags types
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct InterfaceType: TY_INTERFACE_TYPE {
        const UNKNOWN        = 0;
        const RAW            = 1;
        const USB            = 2;
        const ETHERNET       = 4;
        const IEEE80211      = 8;
        const ALL            = 0xffff;
    }
}

impl Display for InterfaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let names: Vec<_> = [InterfaceType::RAW, InterfaceType::USB, InterfaceType::ETHERNET, InterfaceType::IEEE80211].iter().zip([
            "RAW", "USB", "ETH", "WIFI"
        ])
        .filter(|(tp, _)| self.contains(**tp) )
        .map(|(_, name)| name).collect();

        write!(f, "{}", names.join(","))
    }
}

#[repr(transparent)]
pub struct NetInfo(TY_DEVICE_NET_INFO);

pub fn addr_from_arr<const N: usize, const M: usize>(a: [i8; N], n: usize) -> [u8; M] {
    let mut a: [i8; M] = a[..M].try_into().unwrap();
    a.map(|x| x as u8)
}

impl NetInfo {
    pub fn mac(&self) -> MacAddr {
        let s = cstr_to_str(self.0.mac.as_ptr());
        MacAddr::from_str(s).unwrap()
    }
    pub fn ip(&self) -> IpAddr {
        let s = cstr_to_str(self.0.ip.as_ptr());
        IpAddr::from_str(s).unwrap()
    }
    pub fn netmask(&self) -> IpAddr {
        let s = cstr_to_str(self.0.netmask.as_ptr());
        IpAddr::from_str(s).unwrap()
    }
    pub fn gateway(&self) -> Option<IpAddr> {
        let s = cstr_to_str(self.0.gateway.as_ptr());
        if s.is_empty() {
            None
        } else {
            Some(IpAddr::from_str(s).unwrap())
        }
    }
    pub fn broadcast(&self) -> IpAddr {
        let s = cstr_to_str(self.0.broadcast.as_ptr());
        IpAddr::from_str(s).unwrap()
    }
}

impl AsRef<NetInfo> for TY_DEVICE_NET_INFO {
    fn as_ref(&self) -> &NetInfo {
        unsafe { transmute(self) }
    }
}

pub fn ty_init_lib() -> Result<()> {
    chkerr(unsafe {_TYInitLib() })
}

pub fn ty_deinit_lib() -> Result<()> {
    chkerr(unsafe {TYDeinitLib() })
}

pub fn ty_lib_version() -> Result<VersionInfo> {
    let mut ver = std::mem::MaybeUninit::uninit();
    chkerr(unsafe {TYLibVersion(ver.as_mut_ptr())})?;
    Ok(unsafe { VersionInfo(ver.assume_init())})
}

pub fn ty_update_interface_list() -> Result<()> {
    chkerr(unsafe {TYUpdateInterfaceList() })
}

pub fn ty_get_interface_number() -> Result<usize> {
    let mut n: u32 = 0;
    chkerr(unsafe{TYGetInterfaceNumber(&mut n)})?;
    Ok(n as usize)
}

pub fn ty_get_interface_list(mut n: usize) -> Result<Vec<InterfaceInfo>> {
    if n == 0 {
        n = ty_get_interface_number().unwrap();
    }
    let mut out = Vec::<TY_INTERFACE_INFO>::with_capacity(n);
    let mut filled_n = 0;
    unsafe {
        chkerr(TYGetInterfaceList(out.as_mut_ptr(), n as u32, &mut filled_n))?;
        out.set_len(filled_n as usize);
    }
    Ok(out.into_iter().map(|x|  {unsafe{transmute(x)}}).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        const DEV_NR: usize = 4;
        let ver: (i32, i32, i32) = ty_lib_version().unwrap().into();
        assert_eq!(ver, (3, 6, 66));
        ty_init_lib().unwrap();

        ty_update_interface_list().unwrap();
        let n = ty_get_interface_number().unwrap();
        assert_eq!(n, DEV_NR);

        let dev_list = ty_get_interface_list(n).unwrap();
        assert_eq!(dev_list.len(), DEV_NR);
        // dbg!(dev_list);

        assert_eq!(DeviceError::from_repr(1002).unwrap(), DeviceError::NotInited);

        dbg!(ty_error_string(-1001));
    }


}
