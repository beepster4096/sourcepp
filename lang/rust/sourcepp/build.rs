use std::{
    env,
    error::Error,
    fs, iter,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    let sourcepp_path = std::path::absolute(Path::new("../../.."))?; // FIXME
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

    let vendored = Path::new("./vendored");

    // build sourcepp
    let mut cmake = cmake::Config::new(&sourcepp_path);

    cmake.define("SOURCEPP_LIBS_START_ENABLED", "OFF");
    cmake.define("SOURCEPP_BUILD_FROM_RUST_WRAPPER", "ON");

    let target_feature_cfg = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();

    if target_feature_cfg.contains("crt-static") {
        cmake.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded");
    } else {
        cmake.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreadedDLL");
    }
    for lib in enabled_libraries() {
        cmake.define(format!("SOURCEPP_USE_{}", lib.to_uppercase()), "ON");
    }

    let provider = sourcepp_path.join("cmake/VendoredDependencyProvider.cmake");

    if fs::exists(vendored)? {
        cmake.define("CMAKE_PROJECT_TOP_LEVEL_INCLUDES", provider.canonicalize()?);
        cmake.define("SOURCEPP_VENDORED_PATH", std::path::absolute(vendored)?); // canonicalize will kill msvc
    }

    cmake.always_configure(env::var("SOURCEPP_ALWAYS_RECONFIGURE").is_ok());

    cmake.build();

    for lib in fs::read_to_string(out_dir.join("rustc_link_lib.txt"))?.lines() {
        println!("cargo::rustc-link-lib=static:-bundle={lib}");
    }

    for path in fs::read_to_string(out_dir.join("rustc_link_search.txt"))?.lines() {
        println!("cargo::rustc-link-search=native={path}");
    }

    let include_txt = fs::read_to_string(out_dir.join("include.txt"))?;

    // generate bindings
    cxx_build::bridges(["src/bridge.rs"])
        .std("c++20")
        .includes(iter::once("include").chain(include_txt.lines()))
        .file("src/shim/vpkpp.cpp")
        .compile("sourcepp-rust");

    println!("cargo::rerun-if-changed=src/bridge.rs");

    Ok(())
}
