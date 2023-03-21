mod compilation;
mod sapi;
mod stubs;

use crate::fastly_ce::manager::response;

use self::sapi::init_fastly_ce_sapi;
pub use compilation::compile_from_stdin;
use php_sys::*;
use std::ptr::{null_mut, NonNull};

pub fn execute_compiled(op_array: *mut zend_op_array) {
    println!("Started executing PHP");
    unsafe {
        zend_execute(op_array, null_mut());
    };
    log_exceptions();

    // todo: tmp for testing
    let mut response = response();

    response.flush();

    println!("PHP code executed");
}

fn log_exceptions() {
    let mut globals = unsafe { executor_globals };

    let mut exception_ptr = std::ptr::null_mut();
    std::mem::swap(&mut exception_ptr, &mut globals.exception);

    unsafe {
        if let Some(exception_ptr) = exception_ptr.as_mut() {
            let mut exception = NonNull::new_unchecked(exception_ptr);
            zend_exception_error(exception.as_mut(), 1);
        }
    }
}

pub fn init() {
    init_fastly_ce_sapi();
    unsafe {
        php_request_startup();
    }
}

pub use self::stubs::generate_fastly_ce_stubs;
