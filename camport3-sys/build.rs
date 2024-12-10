use std::env;
use std::path::PathBuf;

fn main() {
    let prefix = env::var("CONDA_PREFIX").expect("CONDA_PREFIX not exists!");
    println!("cargo:rustc-link-search={prefix}/lib");
    println!("cargo:rustc-link-lib=tycam");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{prefix}/include"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .impl_debug(true)
        // .wrap_static_fns(true) // TODO
    ;

    for p in ["TYVer.h", "TYApi.h", "TYCoordinateMapper.h", "TYImageProc.h", "TyIsp.h"] {
        builder = builder.allowlist_file(format!("{prefix}/include/camport3/{p}"));
    }

    let bindings = builder.generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    bindings
        .write_to_file("gen/bindings.rs")
        .expect("Couldn't write bindings!");



}
