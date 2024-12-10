use std::{fmt::{Debug, Display}, net::IpAddr, str::FromStr};
use serde::{ser::SerializeStruct, Serialize};
use macaddr::MacAddr;
use bitflags::bitflags;

use crate::utils::cstr_to_str;
use crate::ffi::*;

impl VersionInfo {
    pub fn major(&self) -> u32 {
        self.0.major as u32
    }
    pub fn minor(&self) -> u32 {
        self.0.minor as u32
    }
    pub fn patch(&self) -> u32 {
        self.0.patch as u32
    }
}

impl Serialize for VersionInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer {

        let mut state = serializer.serialize_struct("VersionInfo", 3)?;
        state.serialize_field("major", &self.major())?;
        state.serialize_field("minor", &self.minor())?;
        state.serialize_field("patch", &self.patch())?;
        state.end()
    }
}

impl From<VersionInfo> for (u32, u32, u32) {
    fn from(value: VersionInfo) -> Self {
        (value.major(), value.minor(), value.patch())
    }
}

impl Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}

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
        let t = self.type_();
        if t.contains(InterfaceType::ETHERNET) || t.contains(InterfaceType::IEEE80211) {
            Some(self.0.netInfo.as_ref())
        } else {
            None
        }
    }
}

bitflags! {
    // Attributes can be applied to flags types
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct InterfaceType: u32 {
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



// pub fn addr_from_arr<const N: usize, const M: usize>(a: [i8; N], n: usize) -> [u8; M] {
//     let a: [i8; M] = a[..M].try_into().unwrap();
//     a.map(|x| x as u8)
// }

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

impl UsbInfo {
    pub fn bus(&self) -> i32 {
        self.0.bus
    }
    pub fn addr(&self) -> i32 {
        self.0.addr
    }
}

impl DeviceBaseInfo {
    pub fn iface(&self) -> &InterfaceInfo {
        self.0.iface.as_ref()
    }

    pub fn id(&self) -> &str {
        cstr_to_str(self.0.id.as_ptr())
    }

    pub fn vender_name(&self) -> &str {
        cstr_to_str(self.0.vendorName.as_ptr())
    }

    pub fn user_defined_name(&self) -> &str {
        cstr_to_str(self.0.userDefinedName.as_ptr())
    }

    pub fn model_name(&self) -> &str {
        cstr_to_str(self.0.modelName.as_ptr())
    }

    pub fn hardware_version(&self) -> &VersionInfo {
        self.0.hardwareVersion.as_ref()
    }

    pub fn firmware_version(&self) -> &VersionInfo {
        self.0.firmwareVersion.as_ref()
    }

    pub fn get_net_info(&self) -> Option<&NetInfo> {
        let t = self.iface().type_();
        if t.contains(InterfaceType::ETHERNET) || t.contains(InterfaceType::IEEE80211) {
            Some(unsafe{self.0.__bindgen_anon_1.netInfo.as_ref()}) // better way?
        } else {
            None
        }
    }

    pub fn get_usb_info(&self) -> Option<&UsbInfo> {
        let t = self.iface().type_();
        if t.contains(InterfaceType::USB) {
            Some(unsafe { &self.0.__bindgen_anon_1.usbInfo.as_ref() }) // better way?
        } else {
            None
        }
    }

    pub fn build_hash(&self) -> &str {
        cstr_to_str(self.0.buildHash.as_ptr())
    }

    pub fn config_version(&self) -> &str {
        cstr_to_str(self.0.configVersion.as_ptr())
    }
}

impl InterfaceHandle<'_> {
    pub fn update_device_list(&self) -> Result<()> {
        ty_update_device_list(self)
    }

    pub fn get_device_list(&self, n: usize) -> Result<Vec<DeviceBaseInfo>> {
        ty_get_device_list(self, n)
    }

    pub fn has_device(&self, id: &str) -> Result<bool> {
        ty_has_device(self, id)
    }
}

impl Drop for InterfaceHandle<'_> {
    fn drop(&mut self) {
        ty_close_interface(self).unwrap()
    }
}

// function start here

// TODO serialize Information struct to json like
#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ID: &str = "eth-30:0e:d5:57:c2:ea9b04a8c0";

    fn setup_context() -> Context {
        let ctx = Context::new();
        ctx.update_interface_list();
        let out = ctx.get_interface_list(0);
        assert!(out.len() >= 1);
        ctx
    }

    #[test]
    fn test_device() {
        let ctx = setup_context();

        let ifaceh = ctx.open_interface(VALID_ID).unwrap();
        ifaceh.update_device_list().unwrap();
        let mut _n = 0;
        for dev in ifaceh.get_device_list(0).unwrap() {
            assert!(!dev.id().is_empty());
            _n += 1;
        }

    }
}