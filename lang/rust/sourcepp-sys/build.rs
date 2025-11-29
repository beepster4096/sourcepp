use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    let sourcepp_path = Path::new("../../.."); // FIXME
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let all_libraries = [
        "bsppp", "dmxpp", "fspp", "gamepp", "kvpp", "mdlpp", "sndpp", "steampp", "toolpp",
        "vcryptpp", "vpkpp", "vtfpp",
    ];

    let feature_cfg = env::var("CARGO_CFG_FEATURE").unwrap();
    let features: Vec<_> = feature_cfg.split(",").collect();

    // right now all features are libraries to build,
    // but features might be used to configure building in the future
    let enabled_libraries = || features.iter().filter(|f| all_libraries.contains(f));

    // build sourcepp
    let mut cmake = cmake::Config::new(&sourcepp_path);

    cmake.define("SOURCEPP_LIBS_START_ENABLED", "OFF");
    //cmake.always_configure(false); // FIXME
    for lib in enabled_libraries() {
        cmake.define(format!("SOURCEPP_USE_{}", lib.to_uppercase()), "ON");
    }

    let remote_libs = Path::new("./ext_remote");

    if fs::exists(remote_libs)? {
        cmake.define("SOURCEPP_REMOTE_LIBS_SRC", remote_libs.canonicalize()?);
    }

    let dst = cmake.build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=foo");

    // generate bindings
    let include_dir = sourcepp_path.join("include");
    let include_dir_regex = format!("{}.*", regex::escape(&include_dir.to_string_lossy()));

    let bindings = bindgen::builder()
        .headers(enabled_libraries().map(|lib| {
            include_dir
                .join(format!("{lib}/{lib}.h"))
                .to_string_lossy()
                .into_owned()
        }))
        .clang_args([
            "-x",
            "c++",
            "-std=c++20",
            &format!("-I{}", include_dir.display()),
            &format!("-I{}", sourcepp_path.join("ext/half/include").display()),
            &format!("-I{}", remote_libs.join("tsl_hat_trie/include").display()),
            &format!("-I{}", remote_libs.join("bufferstream/include").display()),
        ])
        .allowlist_file(include_dir_regex)
        //.blocklist_item("std")
        //.blocklist_item("kvpp::.*")
        .enable_cxx_namespaces()
        .respect_cxx_access_specs(true)
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .opaque_type("std::.*")
        .wrap_unsafe_ops(true)
        .derive_debug(false)
        .generate()?;

    bindings.write_to_file(out_dir.join("bindings.rs"))?;

    Ok(())
}
