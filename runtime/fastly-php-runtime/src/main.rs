use std::{cell::RefCell, ptr::null_mut};

use fastly::handle::{client_request_and_body, BodyHandle, ResponseHandle};
use php::{compile_from_stdin, execute_compiled_with_ce};

mod php;
mod util;

thread_local! {
    static OP_ARRAY: RefCell<*mut php_sys::zend_op_array> = RefCell::new(null_mut());
}

pub fn main() {
    fastly::init();
    let (client_req_handle, client_body_handle) = client_request_and_body();
    let res_handle = ResponseHandle::new();
    let res_body_handle = BodyHandle::new();

    OP_ARRAY.with(|op_array| {
        execute_compiled_with_ce(
            *op_array.borrow(),
            client_req_handle,
            client_body_handle,
            res_handle,
            res_body_handle,
        );
    });
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    crate::php::init();

    let op_array = compile_from_stdin();

    OP_ARRAY.with(|op_array_cell| {
        *op_array_cell.borrow_mut() = op_array;
    });
}
