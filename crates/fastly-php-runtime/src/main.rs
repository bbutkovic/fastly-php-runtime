use std::{cell::RefCell, ptr::null_mut};

use php::{compile_from_stdin, execute_compiled, generate_fastly_ce_stubs, init as init_php};

mod fastly_ce;
mod php;
mod util;

thread_local! {
    static OP_ARRAY: RefCell<*mut php_sys::zend_op_array> = RefCell::new(null_mut());
}

pub fn main() {
    fastly::init();

    OP_ARRAY.with(|op_array| {
        execute_compiled(*op_array.borrow());
    });
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    println!("Initializing PHP");

    init_php();

    println!("Loading and compiling PHP from STDIN");

    let op_array = compile_from_stdin();

    OP_ARRAY.with(|op_array_cell| {
        *op_array_cell.borrow_mut() = op_array;
    });

    println!("Code loaded and compiled");
}

#[export_name = "generate_fastly_ce_stubs"]
pub extern "C" fn generate_stubs() {
    let stubs = generate_fastly_ce_stubs();

    print!("{stubs}")
}
