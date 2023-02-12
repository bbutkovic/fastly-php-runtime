// use std::ffi::CString;

use std::ffi::CString;

use fastly::{Error, Request, Response as FastlyResponse};

use php_sys::*;
// mod response;

// use response::Response;

// #[fastly::main]
// fn main(req: Request) -> Result<FastlyResponse, Error> {
//     // Response::initialize();

//     // run_php(req, "phpinfo();");

//     let res = FastlyResponse::from_status(200);
//     // let res = Response::get();
//     Ok(res)
// }

fn main() {
    println!("Hello, world!");

    unsafe { run_php() }
}

unsafe fn run_php() {
    php_embed_module.ub_write = Some(embed_write);
    php_embed_init(0, std::ptr::null_mut());

    zend_eval_string(
        CString::new("echo 'Hello, world!';").unwrap().into_raw(),
        std::ptr::null_mut(),
        CString::new("fastly-php").unwrap().into_raw(),
    );
}

unsafe extern "C" fn embed_write(str: *const ::std::os::raw::c_char, str_length: size_t) -> size_t {
    let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

    println!("a: {}", str);

    str_length
}

// pub fn run_php(req: Request, code: &str) -> Option<String> {
//     let code = CString::new(code).unwrap();

//     unsafe {
//         php::php_embed_module.ub_write = Some(embed_write);

//         php::php_embed_init(0, std::ptr::null_mut());

//         let script = std::ptr::null_mut::<php::zval>();

//         let code_raw = code.into_raw();

//         php::zend_eval_string(
//             code_raw,
//             script,
//             CString::new("fastly-php").unwrap().into_raw(),
//         );

//         php::_convert_to_string(script);

//         println!("return: {:?}", (*(*script).value.str).val);
//     }

//     None
// }

// unsafe extern "C" fn embed_write(
//     str: *const ::std::os::raw::c_char,
//     str_length: php::size_t,
// ) -> php::size_t {
//     let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

//     Response::write(str);

//     str_length
// }

// // TODO
// #[no_mangle]
// extern "C" fn getpid() -> i32 {
//     0
// }

// #[no_mangle]
// extern "C" fn times(_: i32) -> i64 {
//     0
// }

/*

use fastly::{Error, Request, Response as FastlyResponse};
use response::Response;

mod archive;
mod noops;
mod response;
mod runtime;
mod util;

#[fastly::main]
fn main(_req: Request) -> Result<FastlyResponse, Error> {
    hello_world_if_empty();
    println!("req");
    archive::phar::execute_loaded();
    Ok(FastlyResponse::from_body("Hello, world!"))
}

fn hello_world_if_empty() {
    archive::phar::store_from_bytes(bytes::Bytes::from("<?php echo 'Hello, world!'; "));
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    archive::phar::store_from_stdin();
}

*/
