use camport3_rs::Context;

fn main() {
    let ctx = Context::new();
    let ver = ctx.version();
    println!("library version: {ver}");

    ctx.update_interface_list();
    let l = ctx.get_interface_list(0);
    for iface in l {
        println!("==== Interface ===");
        println!("name: {}", iface.name());
        println!("id: {}", iface.id());
        println!("type: {}", iface.type_());
        let netinfo = iface.net_info();
        if let Some(netinfo) = netinfo {
            println!("mac: {}", netinfo.mac());
            println!("ip: {}", netinfo.ip());
            println!("netmask: {}", netinfo.netmask());
            if let Some(gateway) = netinfo.gateway() {
                println!("gateway: {}", gateway);
            }
            println!("broadcast: {}", netinfo.broadcast());
        }
    }


    // assert_eq!(ver, (3, 6, 66));
    // ty_init_lib().unwrap();

    // ty_update_interface_list().unwrap();
    // let n = ty_get_interface_number().unwrap();
    // assert_eq!(n, DEV_NR);

    // let dev_list = ty_get_interface_list(n).unwrap();
    // assert_eq!(dev_list.len(), DEV_NR);
    // // dbg!(dev_list);

    // assert_eq!(DeviceError::from_repr(1002).unwrap(), DeviceError::NotInited);

    // dbg!(ty_error_string(-1001));
}