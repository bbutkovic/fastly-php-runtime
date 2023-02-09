use std::ffi::CString;

use fastly::{Error, Request, Response as FastlyResponse};
mod php;
mod response;

use response::Response;

#[fastly::main]
fn main(req: Request) -> Result<FastlyResponse, Error> {
    Response::initialize();

    run_php(req, "phpinfo();");

    let res = Response::get();
    Ok(res)
}

pub fn run_php(req: Request, code: &str) -> Option<String> {
    let code = CString::new(code).unwrap();

    unsafe {
        php::php_embed_module.ub_write = Some(embed_write);

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

unsafe extern "C" fn embed_write(
    str: *const ::std::os::raw::c_char,
    str_length: php::size_t,
) -> php::size_t {
    let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

    Response::write(str);

    str_length
}

// TODO
#[no_mangle]
extern "C" fn getpid() -> i32 {
    0
}

#[no_mangle]
extern "C" fn times(_: i32) -> i64 {
    0
}
