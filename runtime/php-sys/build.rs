extern crate bindgen;

use std::collections::HashMap;
use std::env::var;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use regex::Regex;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include = out_dir.join("include");

    cp_r("vendor/php", &include);

    let (wasi_sdk_sysroot, compiler) = get_wasi_sdk();

    compile_php(&include, wasi_sdk_sysroot, compiler);

    println!(
        "cargo:rustc-link-search={}",
        include.as_path().to_str().unwrap()
    );

    let wrapper = "wrapper.h".to_string();
    println!("cargo:rustc-link-lib=php");
    println!("cargo:rerun-if-changed={}", wrapper);

    generate_bindings(wrapper, &include, out_dir.join("bindings.rs"));
}

macro_rules! include_flag {
    ($root:expr) => {
        format!("-I{}", $root.clone().to_str().unwrap().to_string())
    };

    ($root:expr, $path:expr) => {
        include_flag!($root.join($path))
    };
}

fn generate_bindings(wrapper: String, sources_root: &PathBuf, out_file: PathBuf) {
    let bindings = bindgen::Builder::default()
        .header(wrapper)
        .clang_arg(include_flag!(sources_root))
        .clang_arg(include_flag!(sources_root, "main"))
        .clang_arg(include_flag!(sources_root, "Zend"))
        .clang_arg(include_flag!(sources_root, "TSRM"))
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

    fs::write(out_file, bindings).unwrap();
}

fn compile_php(source: &PathBuf, wasi_sdk_sysroot: Option<PathBuf>, compiler: Option<PathBuf>) {
    println!("Configuring PHP");
    configure_php(source, wasi_sdk_sysroot, compiler);

    println!("Building PHP");
    build_php(source);
}

fn configure_php(source: &PathBuf, wasi_sdk_sysroot: Option<PathBuf>, compiler: Option<PathBuf>) {
    let mut buildconf = Command::new("./buildconf");
    buildconf.arg("--force");
    buildconf.current_dir(&source);

    if !buildconf.status().unwrap().success() {
        panic!("Failed to run buildconf");
    }

    let mut cflags = vec![
        "-O0".to_string(),
        "-D_WASI_EMULATED_GETPID".to_string(),
        "-D_WASI_EMULATED_SIGNAL".to_string(),
        "-D_WASI_EMULATED_PROCESS_CLOCKS".to_string(),
        "-D_POSIX_SOURCE=1".to_string(),
        "-D_GNU_SOURCE=1".to_string(),
        "-DHAVE_FORK=0".to_string(),
        "-DWASM_WASI".to_string(),
    ];

    let mut ldflags = vec![
        "-lwasi-emulated-getpid".to_string(),
        "-lwasi-emulated-signal".to_string(),
        "-lwasi-emulated-process-clocks".to_string(),
    ];

    if let Some(wasi_sdk_sysroot) = wasi_sdk_sysroot {
        let wasi_sdk_sysroot = wasi_sdk_sysroot.to_str().unwrap();
        let sysroot_flag = format!("--sysroot={}", wasi_sdk_sysroot);
        cflags.push(sysroot_flag.clone());
        ldflags.push(sysroot_flag);
    }

    let mut build_env =
        HashMap::from([("CFLAGS", cflags.join(" ")), ("LDFLAGS", ldflags.join(" "))]);

    if let Some(compiler) = compiler {
        build_env.insert("CC", compiler.to_str().unwrap().to_string());
    }

    let mut configure = Command::new("./configure");

    configure
        .current_dir(&source)
        .arg("--enable-embed=static")
        .arg("--host=wasm32-wasi")
        .arg("--target=wasm32-wasi")
        .arg("--without-libxml")
        .arg("--disable-dom")
        .arg("--without-iconv")
        .arg("--without-openssl")
        .arg("--disable-simplexml")
        .arg("--disable-xml")
        .arg("--disable-xmlreader")
        .arg("--disable-xmlwriter")
        .arg("--without-pear")
        .arg("--disable-opcache")
        .arg("--disable-zend-signals")
        .arg("--without-pcre-jit")
        .arg("--without-sqlite3")
        .arg("--without-pdo-sqlite")
        .arg("--enable-phar=static")
        .arg("--enable-pdo=static")
        .envs(&build_env);

    println!("Running configure: {:?}", configure);

    let success = configure
        .output()
        .unwrap_or_else(|err| panic!("{:?} failed ({})", configure, err))
        .status
        .success();

    if !success {
        panic!("Failed to run configure");
    }
}

fn build_php(source: &PathBuf) {
    let mut build = Command::new("make");
    build.current_dir(&source);
    build.arg("libphp.la");

    println!("Building: {:?}", build);

    let output = build
        .output()
        .unwrap_or_else(|_| panic!("{:?} failed", build));

    println!("Output: {:?}", output);

    if !output.status.success() {
        panic!("Failed to build PHP");
    }
}

fn get_wasi_sdk() -> (Option<PathBuf>, Option<PathBuf>) {
    let sysroot = var("PHP_WASI_SYSROOT")
        .or(var("WASI_SYSROOT"))
        .ok()
        .or_else(|| {
            var("WASI_SDK_PATH")
                .ok()
                .map(|path| format!("{}/share/wasi-sysroot", path))
        });

    let compiler = var("PHP_WASI_COMPILER")
        .or(var("WASI_SDK_COMPILER"))
        .ok()
        .or_else(|| {
            var("WASI_SDK_PATH")
                .ok()
                .map(|path| format!("{}/bin/clang", path))
        });

    (sysroot.map(PathBuf::from), compiler.map(PathBuf::from))
}

// todo
fn cp_r(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    for e in from.as_ref().read_dir().unwrap() {
        let e = e.unwrap();
        let from = e.path();
        let to = to.as_ref().join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            fs::create_dir_all(&to).unwrap();
            cp_r(&from, &to);
        } else {
            // println!("{} => {}", from.display(), to.display());
            fs::copy(&from, &to).unwrap();
        }
    }
}
