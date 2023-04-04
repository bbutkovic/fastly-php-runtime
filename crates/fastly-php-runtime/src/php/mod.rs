// mod compilation;
mod loading;
mod runtime;
mod sapi;
mod stubs;

pub use runtime::Runtime;
pub use stubs::generate_fastly_ce_stubs;

// use crate::fastly_ce::{
//     manager::{request, response},
//     request::RequestHandle,
// };

// // use self::sapi::init_fastly_ce_sapi;
// use compilation::compile_from_stdin as sapi_compile_from_stdin;
// use php_sys::*;
// use std::ptr::{null_mut, NonNull};

// pub fn execute_compiled(op_array: *mut zend_op_array) {
//     println!("Started executing PHP");

//     init_execution_environment();

//     unsafe {
//         zend_execute(op_array, null_mut());
//     };

//     log_exceptions();
//     shutdown_execution_environment();

//     // todo: tmp for testing
//     let mut response = response();

//     response.flush();

//     println!("PHP code executed");
// }

// fn log_exceptions() {
//     let mut globals = unsafe { executor_globals };

//     let mut exception_ptr = std::ptr::null_mut();
//     std::mem::swap(&mut exception_ptr, &mut globals.exception);

//     unsafe {
//         if let Some(exception_ptr) = exception_ptr.as_mut() {
//             let mut exception = NonNull::new_unchecked(exception_ptr);
//             zend_exception_error(exception.as_mut(), 1);
//         }
//     }
// }

// pub fn compile_from_stdin() -> anyhow::Result<*mut zend_op_array> {
//     init_fastly_ce_sapi();

//     unsafe {
//         zend_interned_strings_activate();

//         zend_activate();
//     }

//     let op_array = sapi_compile_from_stdin();

//     unsafe {
//         zend_deactivate();

//         zend_interned_strings_deactivate();

//         php_output_shutdown();

//         zend_interned_strings_dtor();
//         // core_globals_dtor(&core_globals);
//         // gc_globals_dtor();
//     }

//     Ok(op_array)
// }

// fn init_execution_environment() {
//     let mut request = request();

//     unsafe {
//         // initialize_request_info(&mut sapi_globals.request_info, &mut *request);

//         php_request_startup();
//     }
// }

// unsafe fn initialize_request_info(
//     request_info: *mut sapi_request_info,
//     request: &mut RequestHandle,
// ) {
//     let request_method = request.request_method();
//     (*request_info).request_method = request_info_str(request_method);

//     if let Some(request_uri) = request.request_uri() {
//         (*request_info).request_uri = request_info_str(request_uri);
//     }

//     if let Some(query_string) = request.query_string() {
//         (*request_info).query_string = request_info_str(query_string);
//     }
// }

// fn request_info_str(string: String) -> *mut i8 {
//     let string = std::ffi::CString::new(string).unwrap().into_boxed_c_str();

//     &*string as *const _ as *mut i8
// }

// fn shutdown_execution_environment() {
//     unsafe {
//         php_request_shutdown(null_mut());
//     }
// }

// pub use self::stubs::generate_fastly_ce_stubs;
