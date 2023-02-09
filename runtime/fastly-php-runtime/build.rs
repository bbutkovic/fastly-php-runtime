extern crate bindgen;

use std::path::PathBuf;
use std::{env, fs};

use regex::Regex;

fn main() {
    println!("cargo:rustc-link-search=dependencies/lib/");
    println!("cargo:rustc-link-lib=php");
    println!("cargo:rustc-link-lib=clang_rt");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Idependencies/sources/php-7.4.32/")
        .clang_arg("-Idependencies/sources/php-7.4.32/main")
        .clang_arg("-Idependencies/sources/php-7.4.32/Zend")
        .clang_arg("-Idependencies/sources/php-7.4.32/TSRM")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_path.join("bindings.rs"), bindings).unwrap();
}
