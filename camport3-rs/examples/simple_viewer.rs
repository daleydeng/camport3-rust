use camport3_rs::{fmt_ty_interface_type, Context};

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
        println!("type: {}", fmt_ty_interface_type(iface.type_()));
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


}