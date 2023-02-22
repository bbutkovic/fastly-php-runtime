extern crate bindgen;

use std::collections::HashMap;
use std::env::var;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use regex::Regex;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include = out_dir.join("include");

    cp_r("vendor/php", &include);

    let (wasi_sdk_sysroot, compiler, ranlib, ar, nm) = get_wasi_sdk();

    compile_php(&include, &wasi_sdk_sysroot, &compiler, &ranlib, &ar, &nm);

    println!(
        "cargo:rustc-link-search={}",
        include.join("libs").as_path().to_str().unwrap()
    );

    // todo...
    if let Some(libclang_rt) = var("PHP_WASI_LIBCLANG_RT").ok() {
        let libclang_rt_path = PathBuf::from(libclang_rt);
        if libclang_rt_path.exists() {
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
    }

    // todo....
    if let Some(emulators_path) = var("PHP_WASI_EMULATORS_PATH").ok() {
        let emulators_path = PathBuf::from(emulators_path);
        if emulators_path.exists() {
            println!(
                "cargo:rustc-link-search=native={}",
                emulators_path.to_str().unwrap()
            );
        }

        println!("cargo:rustc-link-lib=static=wasi-emulated-getpid");
        println!("cargo:rustc-link-lib=static=wasi-emulated-process-clocks");
    }

    let wrapper = "wrapper.h".to_string();
    println!("cargo:rustc-link-lib=php");
    println!("cargo:rerun-if-changed={}", wrapper);

    generate_bindings(
        wrapper,
        &include,
        out_dir.join("bindings.rs"),
        wasi_sdk_sysroot,
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
    sources_root: &PathBuf,
    out_file: PathBuf,
    wasi_sdk_sysroot: Option<PathBuf>,
) {
    if !var("CLANG_PATH").is_ok() {
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

fn compile_php(
    source: &PathBuf,
    wasi_sdk_sysroot: &Option<PathBuf>,
    compiler: &Option<PathBuf>,
    ranlib: &Option<PathBuf>,
    ar: &Option<PathBuf>,
    nm: &Option<PathBuf>,
) {
    println!("Configuring PHP");
    let build_env = get_build_env(wasi_sdk_sysroot, compiler, ranlib, ar, nm);
    configure_php(source, &build_env);

    println!("Building PHP");
    build_php(source, &build_env);
}

fn configure_php(source: &PathBuf, build_env: &HashMap<String, String>) {
    let mut buildconf = Command::new("./buildconf");
    buildconf.arg("--force");
    buildconf.current_dir(&source);

    if !buildconf.status().unwrap().success() {
        panic!("Failed to run buildconf");
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
        .arg("--with-pic")
        .envs(build_env);

    println!("Running configure: {:?}", configure);

    let output = configure
        .output()
        .unwrap_or_else(|err| panic!("{:?} failed ({})", configure, err));

    // todo....
    if output.status.code().unwrap() != 77 && output.status.code().unwrap() != 0 {
        panic!("Failed to run configure: '{}'", output.status.to_string());
    }
}

fn get_build_env(
    wasi_sdk_sysroot: &Option<PathBuf>,
    compiler: &Option<PathBuf>,
    ranlib: &Option<PathBuf>,
    ar: &Option<PathBuf>,
    nm: &Option<PathBuf>,
) -> HashMap<String, String> {
    let php_debug = env::var_os("PHP_DEBUG").map(|d| d == "1").unwrap_or(false);
    let optimization_level: String = env::var_os("OPT_LEVEL")
        .map_or_else(
            || match php_debug {
                true => Some("0".to_string()),
                false => Some("3".to_string()),
            },
            |l| Some(l.to_str().unwrap().to_string()),
        )
        .unwrap();

    let mut cflags = vec![
        // format!("-O{}", optimization_level).to_string(),
        "-O3".to_string(),
        "-D_WASI_EMULATED_GETPID".to_string(),
        "-D_WASI_EMULATED_SIGNAL".to_string(),
        "-D_WASI_EMULATED_PROCESS_CLOCKS".to_string(),
        "-D_POSIX_SOURCE=1".to_string(),
        "-D_GNU_SOURCE=1".to_string(),
        "-DHAVE_FORK=0".to_string(),
        "-DWASM_WASI".to_string(),
        "-fPIC".to_string(),
    ];

    let mut ldflags = vec![
        "-lwasi-emulated-getpid".to_string(),
        "-lwasi-emulated-signal".to_string(),
        "-lwasi-emulated-process-clocks".to_string(),
        "-static".to_string(),
    ];

    if let Some(wasi_sdk_sysroot) = wasi_sdk_sysroot {
        let wasi_sdk_sysroot = wasi_sdk_sysroot.to_str().unwrap();
        let sysroot_flag = format!("--sysroot={}", wasi_sdk_sysroot);
        cflags.push(sysroot_flag.clone());
        ldflags.push(sysroot_flag);
    }

    if php_debug {
        cflags.push("-g".to_string());
    }

    let mut build_env = HashMap::from([
        ("CFLAGS".to_string(), cflags.join(" ")),
        ("LDFLAGS".to_string(), ldflags.join(" ")),
    ]);

    if let Some(compiler) = compiler {
        build_env.insert("CC".to_string(), compiler.to_str().unwrap().to_string());
    }

    if let Some(ranlib) = ranlib {
        build_env.insert("RANLIB".to_string(), ranlib.to_str().unwrap().to_string());
    }

    if let Some(ar) = ar {
        build_env.insert("AR".to_string(), ar.to_str().unwrap().to_string());
    }

    if let Some(nm) = nm {
        build_env.insert("NM".to_string(), nm.to_str().unwrap().to_string());
    }

    build_env
}

fn build_php(source: &PathBuf, build_env: &HashMap<String, String>) {
    let mut build = Command::new("make");
    build.current_dir(&source);
    if let Some(jobs) = var("NUM_JOBS").ok() {
        build.arg(format!("-j{}", jobs));
    }
    build.arg("libphp.la");
    build.envs(build_env);

    println!("Building: {:?}", build);

    let output = build
        .output()
        .unwrap_or_else(|_| panic!("{:?} failed", build));

    println!("Output: {:?}", output);

    if !output.status.success() {
        panic!("Failed to build PHP");
    }
}

fn get_wasi_sdk() -> (
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
) {
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

    let ranlib = var("PHP_WASI_RANLIB")
        .or(var("WASI_SDK_RANLIB"))
        .ok()
        .or_else(|| {
            var("WASI_SDK_PATH")
                .ok()
                .map(|path| format!("{}/bin/llvm-ranlib", path))
        });

    let ar = var("PHP_WASI_AR").or(var("WASI_SDK_AR")).ok().or_else(|| {
        var("WASI_SDK_PATH")
            .ok()
            .map(|path| format!("{}/bin/llvm-ar", path))
    });

    let nm = var("PHP_WASI_NM").or(var("WASI_SDK_NM")).ok().or_else(|| {
        var("WASI_SDK_PATH")
            .ok()
            .map(|path| format!("{}/bin/llvm-nm", path))
    });

    (
        sysroot.map(PathBuf::from),
        compiler.map(PathBuf::from),
        ranlib.map(PathBuf::from),
        ar.map(PathBuf::from),
        nm.map(PathBuf::from),
    )
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
            fs::copy(&from, &to).unwrap();
        }
    }
}
