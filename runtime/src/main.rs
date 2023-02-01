use std::{ffi::CString, sync::Mutex};

use fastly::{Error, Request, Response as FastlyResponse};
mod php;
mod response;

use response::Response;

const STREAMING: u32 = 1;

#[fastly::main]
fn main(req: Request) -> Result<FastlyResponse, Error> {
    Response::initialize();

    run_php(req, "phpinfo();");

    // let res = Response::new();

    // (*RUNTIME.lock().unwrap()).set_response(res);

    // *RESPONSE.lock().unwrap() = Some(&mut Response::new());

    // *RESPONSE.lock().unwrap() = Some();

    // let res = Response::new();
    // set_current_response(res);

    // ResponseHandle::try_from(5);
    // let mut res = Response::new();

    // run_php(req, &mut res, "phpinfo();");

    // // let body = OUTPUT.lock().unwrap().to_string();

    // Ok(res)
    // Ok(Response::from_body(body))

    let res = Response::get();
    Ok(res)
}

// pub fn init

// fn embed_ub_write(
//     _: i32,
// ) -> unsafe extern "C" fn(str: *const ::std::os::raw::c_char, str_length: php::size_t) -> php::size_t
// {
//     |str: *const ::std::os::raw::c_char, str_length: php::size_t| -> php::size_t {
//         let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

//         *OUTPUT.lock().unwrap() += str;

//         0
//     }
// }

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
