use std::{
    collections::HashMap,
    env::{self, var},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn setup_for_compilation(source: impl AsRef<Path>, dest: impl AsRef<Path>) {
    for e in source.as_ref().read_dir().unwrap() {
        let e = e.unwrap();
        let from = e.path();
        let to = dest.as_ref().join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            fs::create_dir_all(&to).unwrap();
            setup_for_compilation(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

pub fn get_build_env(
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

pub fn get_wasi_sdk() -> (
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

pub fn configure_php(
    source: &PathBuf,
    build_env: &HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let mut buildconf = Command::new("./buildconf");
    buildconf.arg("--force");
    buildconf.current_dir(&source);

    if !buildconf.status().unwrap().success() {
        return Err(anyhow::anyhow!("buildconf failed"));
    }

    let mut configure = Command::new("./configure");

    configure
        .current_dir(&source)
        .arg("--enable-embed=static")
        .arg("--host=wasm32-wasi")
        .arg("--target=wasm32-wasi")
        .arg(format!("--prefix={}", source.to_string_lossy()))
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

    Ok(())
}
