use serde::de::value::Error;
use thiserror::Error;
use bytemuck::TransparentWrapper;
use strum_macros::FromRepr;
use serde::Serialize;
use std::fmt::Display;
use std::{ffi::CString, marker::PhantomData, mem::transmute, ptr};
use camport3_sys::*;
use crate::utils::cstr_to_str;

#[derive(Error, Serialize, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(i32)] // TY_STATUS
pub enum ErrorCode {
    #[error("error")]
    ERROR = TY_STATUS_ERROR,
    #[error("not inited")]
    NotInited = TY_STATUS_NOT_INITED,
    #[error("not implemented")]
    NotImplemented = TY_STATUS_NOT_IMPLEMENTED,
    #[error("not permitted")]
    NotPermitted = -1004,
    #[error("device error")]
    DeviceError = -1005,
    #[error("invalid parameter")]
    InvalidParameter = -1006,
    #[error("invalid handle")]
    InvalidHandle = -1007,
    #[error("invalid component")]
    InvalidComponent = -1008,
    #[error("invalid feature")]
    InvalidFeature = -1009,
    #[error("wrong type")]
    WrongType = -1010,
    #[error("wrong size")]
    WrongSize = -1011,
    #[error("out of memory")]
    OutOfMemory = -1012,
    #[error("out of range")]
    OutOfRange = -1013,
    #[error("timeout")]
    TIMEOUT = -1014,
    #[error("wrong mode")]
    WrongMode = -1015,
    #[error("busy")]
    Busy = -1016,
    #[error("idle")]
    Idle = -1017,
    #[error("no data")]
    NoData = -1018,
    #[error("no buffer")]
    NoBuffer = -1019,
    #[error("null pointer")]
    NullPointer = -1020,
    #[error("readonly feature")]
    ReadonlyFeature = -1021,
    #[error("invalid descriptor")]
    InvalidDescriptor = -1022,
    #[error("invalid interface")]
    InvalidInterface = -1023,
    #[error("foreware error")]
    FirmwareError = -1024,
    #[error("dev error permission")]
    DevEperm = -1,
    #[error("dev error io")]
    DevEio = -5,
    #[error("dev error no memory")]
    DevEnomem = -12,
    #[error("dev error busy")]
    DevEbusy = -16,
    #[error("dev error invalid")]
    DevEinval = -22,
}

#[derive(Error, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceError {
    pub errcode: ErrorCode,
    pub firmware_errcode: Option<u32>,
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.errcode)
    }
}

impl From<i32> for DeviceError {
    fn from(value: i32) -> Self {
        DeviceError {
            errcode: ErrorCode::from_repr(value).unwrap(),
            firmware_errcode: None,
        }

    }
}

pub type Result<T> = std::result::Result<T, DeviceError>;

fn chkerr(status: TY_STATUS) -> Result<()> {
    if status != TY_STATUS_LIST::TY_STATUS_OK as TY_STATUS {
        Err(status.into())
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, TransparentWrapper)]
#[repr(transparent)]
pub struct Wrapper<T>(pub T);

pub type VersionInfo = Wrapper<TY_VERSION_INFO>;
pub type InterfaceInfo = Wrapper<TY_INTERFACE_INFO>;
pub type NetInfo = Wrapper<TY_DEVICE_NET_INFO>;
pub type UsbInfo = Wrapper<TY_DEVICE_USB_INFO>;
pub type DeviceBaseInfo = Wrapper<TY_DEVICE_BASE_INFO>;

#[derive(Debug)]
pub struct Context(pub(crate) PhantomData<()>);

impl Drop for Context {
    fn drop(&mut self) {
        ty_deinit_lib().unwrap()
    }
}

impl Context {
    pub fn new() -> Self {
        ty_init_lib().unwrap();
        Self(PhantomData)
    }

    pub fn error_string(&self, status: i32) -> &str {
        ty_error_string(status)
    }

    pub fn version(&self) -> VersionInfo {
        ty_lib_version().unwrap()
    }

    pub fn update_interface_list(&self) {
        ty_update_interface_list().unwrap()
    }

    pub fn get_interface_number(&self) -> usize {
        ty_get_interface_number().unwrap()
    }

    pub fn get_interface_list(&self, n: usize) -> Vec<InterfaceInfo> {
        ty_get_interface_list(n).unwrap()
    }

    pub fn open_interface(&self, id: &str) -> Result<InterfaceHandle> {
        ty_open_interface(self, id)
    }

    pub fn has_interface(&self, id: &str) -> bool {
        ty_has_interface(id).unwrap()
    }
}

#[derive(Debug)]
pub struct InterfaceHandle<'ctx> {
    handle: TY_INTERFACE_HANDLE,
    pub ctx: &'ctx Context,
}

impl Drop for InterfaceHandle<'_> {
    fn drop(&mut self) {
        ty_close_interface(self)
    }
}

#[derive(Debug)]
pub struct DeviceHandle<'iface, 'ctx> {
    handle: TY_DEV_HANDLE,
    pub iface: &'iface InterfaceHandle<'ctx>,
}

impl Drop for DeviceHandle<'_, '_> {
    fn drop(&mut self) {
        ty_close_device(self, false)
    }
}

pub(crate) fn ty_error_string(status: TY_STATUS) -> &'static str {
    cstr_to_str(unsafe{TYErrorString(status)})
}

pub(crate) fn ty_init_lib() -> Result<()> {
    chkerr(unsafe {_TYInitLib() })
}

pub(crate) fn ty_deinit_lib() -> Result<()> {
    chkerr(unsafe {TYDeinitLib() })
}

pub(crate) fn ty_lib_version() -> Result<VersionInfo> {
    let mut out = std::mem::MaybeUninit::uninit();
    let out = unsafe {
        chkerr(TYLibVersion(out.as_mut_ptr()))?;
        out.assume_init()
    };
    Ok(TransparentWrapper::wrap(out))
}

pub(crate) fn ty_update_interface_list() -> Result<()> {
    chkerr(unsafe {TYUpdateInterfaceList() })
}

pub(crate) fn ty_get_interface_number() -> Result<usize> {
    let mut n: u32 = 0;
    chkerr(unsafe{TYGetInterfaceNumber(&mut n)})?;
    Ok(n as usize)
}

pub(crate) fn ty_get_interface_list(n: usize) -> Result<Vec<InterfaceInfo>> {
    let n = if n == 0 {
        ty_get_interface_number().unwrap()
    } else {
        n
    };
    if n == 0 {
        return Ok(Vec::new())
    }

    let mut out = Vec::<TY_INTERFACE_INFO>::with_capacity(n);
    let mut filled_n = 0;
    unsafe {
        chkerr(TYGetInterfaceList(out.as_mut_ptr(), n as u32, &mut filled_n))?;
        out.set_len(filled_n as usize);
        Ok(transmute(out))
    }
}

pub(crate) fn ty_has_interface(id: &str) -> Result<bool> {
    let mut out = false;
    let id = CString::new(id).unwrap();
    let id = id.as_ptr();
    chkerr(unsafe{TYHasInterface(id, &mut out)})?;
    Ok(out)
}

pub(crate) fn ty_open_interface<'ctx>(ctx: &'ctx Context, id: &str) -> Result<InterfaceHandle<'ctx>> {
    let mut out = ptr::null_mut();
    let id = CString::new(id).unwrap();
    let id = id.as_ptr();
    chkerr(unsafe{
        TYOpenInterface(id, &mut out)
    })?;
    if out.is_null() {
        panic!("handle cannot be NULL!");
    }
    Ok(InterfaceHandle{
        handle: out,
        ctx,
    })
}

pub(crate) fn ty_close_interface(h: &InterfaceHandle) {
    chkerr(unsafe{
        TYCloseInterface(h.handle)
    }).unwrap()
}

pub(crate) fn ty_update_device_list(h: &InterfaceHandle) -> Result<()> {
    chkerr(unsafe{
        TYUpdateDeviceList(h.handle)
    })
}

pub(crate) fn ty_get_device_number(h: &InterfaceHandle) -> Result<usize> {
    let mut n: u32 = 0;
    chkerr(unsafe{TYGetDeviceNumber(h.handle, &mut n)})?;
    Ok(n as usize)
}

pub(crate) fn ty_get_device_list(h: &InterfaceHandle, mut n: usize) -> Result<Vec<DeviceBaseInfo>> {
    if n == 0 {
        n = ty_get_device_number(h).unwrap();
    }
    if n == 0 {
        return Ok(Vec::new())
    }

    let mut out = Vec::<TY_DEVICE_BASE_INFO>::with_capacity(n);
    let mut filled_n = 0;
    unsafe {
        chkerr(TYGetDeviceList(h.handle, out.as_mut_ptr(), n as u32, &mut filled_n))?;
        out.set_len(filled_n as usize);
        Ok(transmute(out))
    }
}

pub(crate) fn ty_has_device(h: &InterfaceHandle, id: &str) -> Result<bool> {
    let mut out = false;
    let id = CString::new(id).unwrap();
    let id = id.as_ptr();
    chkerr(unsafe{TYHasDevice(h.handle, id, &mut out)})?;
    Ok(out)
}

pub(crate) fn ty_close_device(h: &DeviceHandle, reboot: bool) {
    chkerr(unsafe{
        TYCloseDevice(h.handle, reboot)
    }).unwrap()
}

pub(crate) fn ty_open_device<'iface, 'ctx>(h: &'iface InterfaceHandle<'ctx>, id: &str) -> Result<DeviceHandle<'iface, 'ctx>> {
    let mut out = DeviceHandle{
        handle: ptr::null_mut(),
        iface: h,
    };
    let mut err_code: TY_FW_ERRORCODE = 0;
    let id = CString::new(id).unwrap();
    let id = id.as_ptr();
    chkerr(unsafe{
        TYOpenDevice(h.handle, id, &mut out.handle, &mut err_code)
    })?;
    if out.handle.is_null() {
        return Err(DeviceError { errcode: ErrorCode::DevEinval, firmware_errcode: None })
    }

    if err_code != 0 {
        return Err(DeviceError {
            errcode: ErrorCode::DeviceError,
            firmware_errcode: Some(err_code),
        })
    } else {
        Ok(out)
    }

}

pub(crate) fn ty_open_device_with_ip<'iface, 'ctx>(h: &'iface InterfaceHandle<'ctx>, ip: &str) -> Result<DeviceHandle<'iface, 'ctx>> {
    let mut out = DeviceHandle{
        handle: ptr::null_mut(),
        iface: h,
    };
    let ip = CString::new(ip).unwrap();
    let ip: *const i8 = ip.as_ptr();
    chkerr(unsafe{
        TYOpenDeviceWithIP(h.handle, ip, &mut out.handle)
    })?;
    if out.handle.is_null() {
        return Err(DeviceError { errcode: ErrorCode::DevEinval, firmware_errcode: None })
    }
    Ok(out)
}

// TYGetDeviceInterface, already implemented struct DeviceHandle


#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ID: &str = "eth-30:0e:d5:57:c2:ea9b04a8c0";

    #[test]
    fn test_basics() {
        assert_eq!(ErrorCode::from_repr(-1002).unwrap(), ErrorCode::NotInited);

        const DEV_NR: usize = 4;

        let ty_ver =  ty_lib_version().unwrap();
        let ver: (u32, u32, u32) = ty_ver.clone().into();
        assert_eq!(ver, (3, 6, 66));
        // dbg!(ty_error_string(-1002));
        ty_init_lib().unwrap();
        assert_eq!(ty_error_string(-1002), "not initialized");

        ty_update_interface_list().unwrap();
        let n = ty_get_interface_number().unwrap();
        assert_eq!(n, DEV_NR);

        let dev_list = ty_get_interface_list(n).unwrap();
        assert_eq!(dev_list.len(), DEV_NR);

        assert_eq!(ty_has_interface("not exists id").unwrap(), false);
        assert_eq!(ty_has_interface(VALID_ID).unwrap(), true);

        let s = serde_yaml::to_string(&ty_ver).unwrap();
        assert_eq!(s, "major: 3\nminor: 6\npatch: 66\n");
    }
}