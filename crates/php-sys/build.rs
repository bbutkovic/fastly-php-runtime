extern crate bindgen;

use std::env::var;
use std::path::{Path, PathBuf};
use std::{env, fs};

use regex::Regex;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let php_wasi_sdk_sysroot = var("PHP_WASI_SDK_SYSROOT").unwrap();
    let php_src_root = var("PHP_SRC_ROOT").ok();
    let php_libphp_path = var("PHP_LIBPHP_PATH").ok();
    let php_libclang_rt_path = var("PHP_LIBCLANG_RT_PATH").ok();
    let php_wasi_emulators_path = var("PHP_WASI_EMULATORS_PATH").ok();

    // link libphp
    if let Some(libphp_path) = php_libphp_path {
        println!("cargo:rustc-link-lib=php");
        println!("cargo:rustc-link-search={}", libphp_path);
    }

    // link libclang_rt
    if let Some(libclang_rt_path) = php_libclang_rt_path {
        let libclang_rt_path = PathBuf::from(libclang_rt_path);
        if libclang_rt_path.is_file() {
            let libclang_rt_filename = libclang_rt_path.file_name().unwrap().to_str().unwrap();
            println!("cargo:rustc-link-lib=static=:{}", libclang_rt_filename);
            println!(
                "cargo:rustc-link-search=native={}",
                libclang_rt_path.parent().unwrap().to_str().unwrap()
            );
        } else {
            println!("cargo:rustc-link-lib=static=clang_rt");
            println!(
                "cargo:rustc-link-search=native={}",
                libclang_rt_path.to_str().unwrap()
            );
        }
    }

    // link emulated getpid and process clocks
    if let Some(wasi_emulators_path) = php_wasi_emulators_path {
        let wasi_emulators_path = PathBuf::from(wasi_emulators_path);
        if wasi_emulators_path.exists() {
            println!(
                "cargo:rustc-link-search=native={}",
                wasi_emulators_path.to_str().unwrap()
            );
        }

        println!("cargo:rustc-link-lib=static=wasi-emulated-getpid");
        println!("cargo:rustc-link-lib=static=wasi-emulated-process-clocks");
    }

    // generate bindings
    let wrapper = "wrapper.h".to_string();
    println!("cargo:rerun-if-changed={}", wrapper);
    generate_bindings(
        wrapper,
        &PathBuf::from(php_src_root.unwrap()),
        out_dir.join("bindings.rs"),
        Some(PathBuf::from(php_wasi_sdk_sysroot)),
    );
}

macro_rules! include_flag {
    ($root:expr) => {
        format!("-I{}", $root.clone().to_str().unwrap().to_string())
    };

    ($root:expr, $path:expr) => {
        include_flag!($root.join($path))
    };
}

fn generate_bindings(
    wrapper: String,
    sources_root: &Path,
    out_file: PathBuf,
    wasi_sdk_sysroot: Option<PathBuf>,
) {
    // fix for newer bindgen versions that pass --target parameter to clang
    if var("CLANG_PATH").is_err() {
        if let Some(wasi_sdk_sysroot) = wasi_sdk_sysroot {
            std::env::set_var(
                "CLANG_PATH",
                format!("{}/bin/clang", wasi_sdk_sysroot.to_str().unwrap()),
            );
        }
    }

    let bindings = bindgen::Builder::default()
        .header(wrapper)
        .clang_arg(include_flag!(sources_root))
        .clang_arg(include_flag!(sources_root, "main"))
        .clang_arg(include_flag!(sources_root, "Zend"))
        .clang_arg(include_flag!(sources_root, "TSRM"))
        .blocklist_type("FP_NAN")
        .blocklist_type("FP_INFINITE")
        .blocklist_type("FP_ZERO")
        .blocklist_type("FP_SUBNORMAL")
        .blocklist_type("FP_NORMAL")
        .blocklist_type("max_align_t")
        .derive_default(true)
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks)) // causes us to rebuild on every build
        .generate()
        .expect("Unable to generate bindings");

    let re = Regex::new(r"pub const FP_(.*): _bindgen_").unwrap();
    let bindings: String = bindings
        .to_string()
        .lines()
        .collect::<Vec<&str>>()
        .iter()
        .map(|line| match re.is_match(line) {
            true => line.replace("pub const", "// pub const"),
            false => line.to_string(),
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(out_file, bindings).unwrap();
}
