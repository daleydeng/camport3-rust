use bytemuck::TransparentWrapper;
use std::{fmt::{Debug, Display}, net::IpAddr, str::FromStr};
use serde::{ser::SerializeStruct, Serialize};
use macaddr::MacAddr;
use camport3_sys::*;

use crate::utils::{bit_is_set, cstr_to_str};
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

    pub fn type_(&self) -> TY_INTERFACE_TYPE {
        self.0.type_
    }

    pub fn net_info(&self) -> Option<&NetInfo> {
        use TY_INTERFACE_TYPE_LIST::*;
        let t = self.type_();

        if bit_is_set(t, TY_INTERFACE_ETHERNET)
        || bit_is_set(t, TY_INTERFACE_IEEE80211) {
            Some(TransparentWrapper::wrap_ref(&self.0.netInfo))
        } else {
            None
        }
    }
}

pub fn fmt_ty_interface_type(self_: TY_INTERFACE_TYPE) -> String {
    use TY_INTERFACE_TYPE_LIST::*;
    let names: Vec<_> = [
        TY_INTERFACE_RAW,
        TY_INTERFACE_USB,
        TY_INTERFACE_ETHERNET,
        TY_INTERFACE_IEEE80211].iter().zip([
        "RAW", "USB", "ETH", "WIFI"
    ])
    .filter(|(tp, _)| bit_is_set(self_, **tp) )
    .map(|(_, name)| name).collect();

    names.join(",")
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
        TransparentWrapper::wrap_ref(&self.0.iface)
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
        TransparentWrapper::wrap_ref(&self.0.hardwareVersion)
    }

    pub fn firmware_version(&self) -> &VersionInfo {
        TransparentWrapper::wrap_ref(&self.0.firmwareVersion)
    }

    pub fn get_net_info(&self) -> Option<&NetInfo> {
        let t = self.iface().type_();
        if bit_is_set(t, TY_INTERFACE_TYPE_LIST::TY_INTERFACE_ETHERNET)
        || bit_is_set(t, TY_INTERFACE_TYPE_LIST::TY_INTERFACE_IEEE80211) {
            Some(TransparentWrapper::wrap_ref(unsafe{&self.0.__bindgen_anon_1.netInfo}))
        } else {
            None
        }
    }

    pub fn get_usb_info(&self) -> Option<&UsbInfo> {
        let t = self.iface().type_();
        if bit_is_set(t, TY_INTERFACE_TYPE_LIST::TY_INTERFACE_USB) {
            Some(TransparentWrapper::wrap_ref(unsafe{&self.0.__bindgen_anon_1.usbInfo}))
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

    pub fn open_device(&self, id: &str) -> Result<DeviceHandle> {
        ty_open_device(self, id)
    }

    pub fn open_device_with_ip(&self, ip: &str) -> Result<DeviceHandle> {
        ty_open_device_with_ip(self, ip)
    }

}


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

        let iface: InterfaceHandle<'_> = ctx.open_interface(VALID_ID).unwrap();
        iface.update_device_list().unwrap();

        let mut dev_ids = Vec::new();
        for dev in iface.get_device_list(0).unwrap() {
            assert!(!dev.id().is_empty());
            dev_ids.push(dev.id().to_owned());
        }

        assert!(dev_ids.len() > 0);
        let dev_id = &dev_ids[0];
        let dev = iface.open_device(dev_id).unwrap();

    }
}