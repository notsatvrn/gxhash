extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {
    // generate seed for GxBuildHasher RNG if entropy isn't available
    let entropy = cfg!(target_feature = "rdrand") || cfg!(feature = "std");
    if entropy {
        println!("cargo:rustc-cfg=entropy")
    } else {
        let out_dir = std::env::var_os("OUT_DIR").unwrap();
        let seed_path = std::path::Path::new(&out_dir).join("seed.rs");

        use rand::RngCore;
        let mut seed = [0u8; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut seed);

        let mut string = "const SEED: [u8; 32] = [".to_owned();
        string.push_str(&seed.into_iter().map(|i| i.to_string()).collect::<Vec<String>>().join(","));
        string.push_str("];");
        std::fs::write(seed_path, &string).unwrap();
    }

    if version_meta().unwrap().channel == Channel::Nightly
    && cfg!(target_arch = "x86_64")
    && cfg!(target_feature = "avx2")
    && cfg!(target_feature = "vaes") {
        println!("cargo:rustc-cfg=hybrid");
    }
}