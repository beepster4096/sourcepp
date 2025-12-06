use std::{
    env,
    error::Error,
    fs,
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

    // build sourcepp
    let mut cmake = cmake::Config::new(&sourcepp_path);

    cmake.define("SOURCEPP_LIBS_START_ENABLED", "OFF");
    //cmake.always_configure(false); // FIXME
    for lib in enabled_libraries() {
        cmake.define(format!("SOURCEPP_USE_{}", lib.to_uppercase()), "ON");
    }

    let vendored = Path::new("./vendored");
    let provider = sourcepp_path.join("cmake/VendoredDependencyProvider.cmake");

    if fs::exists(vendored)? {
        cmake.define("CMAKE_PROJECT_TOP_LEVEL_INCLUDES", provider.canonicalize()?);
        cmake.define("SOURCEPP_VENDORED_PATH", std::path::absolute(vendored)?); // canonicalize will kill msvc
    }

    cmake.generator("Ninja");

    let dst = cmake.build();

    println!("cargo:rustc-link-search=native={}", dst.join("build").display());
    println!("cargo:rustc-link-lib=static=sourcepp");

    // generate bindings
    cxx_build::bridges(["src/bridge.rs"])
        .std("c++20")
        .includes([
            PathBuf::from("include"),
            sourcepp_path.join("include"),
            sourcepp_path.join("ext/half/include"),
            vendored.join("tsl_hat_trie/include"),
            vendored.join("bufferstream/include"),
        ])
        .file("src/shim/vpkpp.cpp")
        .compile("sourcepp-rust");

    println!("cargo:rerun-if-changed=src/bridge.rs");

    Ok(())
}
