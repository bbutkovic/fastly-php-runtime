use std::io::{stdin, Read};

use bytes::Bytes;
use php::generate_fastly_ce_stubs;

use crate::php::Runtime;
use once_cell::sync::Lazy;
use std::sync::Mutex;

mod fastly_ce;
mod php;
mod util;

static INSTANCE: Lazy<Mutex<Runtime>> = Lazy::new(|| Mutex::new(Runtime::new()));

pub fn main() {
    #[cfg(debug_assertions)]
    println!("Got request");

    fastly::init();

    INSTANCE
        .lock()
        .unwrap()
        .exec()
        .expect("runtime execution fail");
}

#[export_name = "wizer.initialize"]
pub extern "C" fn load_code() {
    #[cfg(debug_assertions)]
    println!("Loading PHP code from STDIN");

    let code: Bytes = stdin().bytes().map(|b| b.unwrap()).collect();

    INSTANCE.lock().unwrap().load(code);
    println!("Code loaded");
}

#[export_name = "generate_fastly_ce_stubs"]
pub extern "C" fn generate_stubs() {
    let stubs = generate_fastly_ce_stubs();

    print!("{stubs}")
}
