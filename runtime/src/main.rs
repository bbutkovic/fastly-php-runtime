use std::ffi::CString;

use fastly::{Error, Request, Response};
mod php;

#[fastly::main]
fn main(_req: Request) -> Result<Response, Error> {
    run_php("phpinfo();");

    Ok(Response::from_body("Hello, World!"))
}

pub fn run_php(code: &str) -> Option<String> {
    let code = CString::new(code).unwrap();

    unsafe {
        php::php_embed_init(0, std::ptr::null_mut());

        let script = std::ptr::null_mut::<php::zval>();

        let code_raw = code.into_raw();

        php::zend_eval_string(
            code_raw,
            script,
            CString::new("fastly-php").unwrap().into_raw(),
        );

        php::_convert_to_string(script);

        println!("return: {:?}", (*(*script).value.str).val);
    }

    None
}

#[no_mangle]
extern "C" fn getpid() -> i32 {
    0
}

#[no_mangle]
extern "C" fn times(_: i32) -> i64 {
    0
}
