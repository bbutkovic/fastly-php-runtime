use anyhow::{bail, Context};
use std::{env, path::PathBuf};
use wasmtime::{Engine, Func, Module, Store};
use wasmtime_wasi::{add_to_linker, sync::WasiCtxBuilder, WasiCtx};

use crate::dummy::dummy_linker;

mod dummy;

fn main() -> anyhow::Result<()> {
    let runtime = env::args().nth(1).unwrap_or("runtime.wasm".to_string());

    let runtime_file = PathBuf::from(runtime.clone());

    if !runtime_file.is_file() {
        bail!("{} does not exist or is not a file.", runtime);
    }

    eprintln!("Generating stubs for {}...", runtime);

    generate_stubs(&runtime_file, "generate_fastly_ce_stubs")?;

    Ok(())
}

fn generate_stubs(runtime: &PathBuf, func_name: &str) -> anyhow::Result<()> {
    let (generate, store) = get_generation_func_from_runtime(runtime, func_name)?;

    generate.call(store, &[], &mut [])
}

fn get_generation_func_from_runtime(
    runtime: &PathBuf,
    func_name: &str,
) -> anyhow::Result<(Func, Store<WasiCtx>)> {
    let engine = Engine::default();

    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(&engine, wasi);

    let module = Module::from_file(&engine, runtime)?;
    let mut linker = dummy_linker(&mut store, &module)?;

    add_to_linker(&mut linker, |s| s)?;

    let execution_instance = linker.instantiate(&mut store, &module)?;

    execution_instance
        .get_func(&mut store, func_name)
        .context("generation function not exported")
        .map(move |func| (func, store))
}
