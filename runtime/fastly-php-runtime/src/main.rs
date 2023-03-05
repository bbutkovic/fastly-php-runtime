use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    ptr::null_mut,
};

use fastly::{Error, Request, Response as FastlyResponse};
use php::compile_from_stdin;

use crate::php::execute_compiled;
use bytes::Bytes;

mod php;
mod util;

thread_local! {
    static OP_ARRAY: RefCell<*mut php_sys::zend_op_array> = RefCell::new(null_mut());
}

#[fastly::main]
fn main(req: Request) -> Result<FastlyResponse, Error> {
    OP_ARRAY.with(|op_array| {
        execute_compiled(*op_array.borrow());
    });

    let res = FastlyResponse::from_status(200);
    Ok(res)
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    crate::php::init();

    let op_array = compile_from_stdin();

    OP_ARRAY.with(|op_array_cell| {
        *op_array_cell.borrow_mut() = op_array;
    });
}
