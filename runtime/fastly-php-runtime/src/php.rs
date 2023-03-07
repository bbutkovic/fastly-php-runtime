use core::slice;
use std::{
    io::{self, Read},
    ptr::{null_mut, NonNull},
    sync::Mutex,
};

use bytes::Bytes;
use ext_php_rs;
use fastly_ce_module;
use lazy_static::lazy_static;
use php_sys::*;

use crate::util::*;

lazy_static! {
    static ref STDIN: Mutex<Bytes> = Mutex::new(Bytes::new());
}

pub fn execute_compiled(op_array: *mut zend_op_array) {
    println!("running");
    unsafe { zend_execute(op_array, null_mut()) };

    print_exception();

    println!("php code ran");
}

fn print_exception() {
    let mut globals = unsafe { executor_globals };

    let mut exception_ptr = std::ptr::null_mut();
    std::mem::swap(&mut exception_ptr, &mut globals.exception);

    let mut exception = unsafe { NonNull::new_unchecked(exception_ptr.as_mut().unwrap()) };
    println!("exception: {:?}", exception);

    unsafe { zend_exception_error(exception.as_mut(), 1) };
}

pub fn init() {
    unsafe {
        php_embed_module.startup = Some(php_embed_startup);
        php_embed_module.ub_write = Some(embed_write);
        // php_embed_module.sapi_error
        php_embed_init(0, null_mut());
    }
}

// todo: errors
// ZEND_API ZEND_COLD void zend_error(int type, const char *format, ...) {
// 	zend_string *filename;
// 	uint32_t lineno;
// 	va_list args;

// 	get_filename_lineno(type, &filename, &lineno);
// 	va_start(args, format);
// 	zend_error_va_list(type, filename, lineno, format, args);
// 	va_end(args);
// }

// workaround for loading the fastly-ce module, todo: implement our own sapi
unsafe extern "C" fn php_embed_startup(
    php_sapi_module: *mut _sapi_module_struct,
) -> ::std::os::raw::c_int {
    let ce_module_entry = fastly_ce_module::get_module();

    php_module_startup(php_sapi_module, convert(ce_module_entry), 1);

    0 as ::std::os::raw::c_int
}
// unsafe extern "C" fn php_embed_startup() {}
// static int php_embed_startup(sapi_module_struct *sapi_module)
// {
// 	if (php_module_startup(sapi_module, NULL, 0) == FAILURE) {
// 		return FAILURE;
// 	}
// 	return SUCCESS;
// }

fn convert(bv: *mut ext_php_rs::ffi::_zend_module_entry) -> *mut _zend_module_entry {
    unsafe { &mut *(bv as *mut ext_php_rs::ffi::_zend_module_entry as *mut _zend_module_entry) }
}

pub fn compile_from_stdin() -> *mut zend_op_array {
    println!("loading and compiling");

    (*STDIN.lock().unwrap()) = io::stdin().bytes().map(|b| b.unwrap()).collect();

    let compile_file = unsafe { zend_compile_file.unwrap() };
    let init_string = unsafe { zend_string_init_interned.unwrap() };

    let filename = cstr!("test.php");
    let filename_len = filename.to_str().unwrap().len();

    let primary_file: zend_file_handle = zend_file_handle {
        handle: _zend_file_handle__bindgen_ty_1 {
            stream: zend_stream {
                reader: Some(stdin_reader),
                fsizer: Some(stdin_fsizer),
                closer: Some(stdin_closer),
                isatty: 0,
                handle: std::ptr::null_mut(),
            },
        },
        filename: unsafe { init_string(filename.as_ptr(), filename_len, true) },
        opened_path: std::ptr::null_mut(),
        type_: zend_stream_type_ZEND_HANDLE_STREAM as u8,
        primary_script: true,
        in_list: false,
        buf: std::ptr::null_mut(),
        len: 0,
    };

    let primary = Box::into_raw(Box::new(primary_file));

    let op_array = unsafe { compile_file(primary, 8) };

    op_array
}

// unsafe fn run_php_from_file() {
//     php_embed_module.ub_write = Some(embed_write);
//     php_embed_init(0, std::ptr::null_mut());

//     // php_embed_module.ub_write = Some(embed_write);

//     let init_string = zend_string_init_interned.unwrap();

//     let filename = cstr!("test.phar");

//     let filename_len = filename.to_str().unwrap().len();

//     let primary_file: zend_file_handle = zend_file_handle {
//         handle: _zend_file_handle__bindgen_ty_1 {
//             stream: zend_stream {
//                 reader: Some(embed_phar_reader),
//                 fsizer: Some(embed_phar_fsizer),
//                 closer: Some(embed_phar_closer),
//                 isatty: 0,
//                 handle: std::ptr::null_mut(),
//             },
//         },
//         filename: init_string(filename.as_ptr(), filename_len as u32, true),
//         opened_path: std::ptr::null_mut(),
//         type_: zend_stream_type_ZEND_HANDLE_STREAM as u8,
//         primary_script: true,
//         in_list: false,
//         buf: std::ptr::null_mut(),
//         len: 0,
//     };

//     let primary = Box::new(primary_file);

//     println!("executing primary file:");
//     let ret = zend_execute_scripts(8, std::ptr::null_mut(), 1, primary);
//     println!("finished: {}", ret);
// }

#[no_mangle]
unsafe extern "C" fn embed_write(str: *const ::std::os::raw::c_char, str_length: usize) -> usize {
    let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

    println!("ub_write: {}", str);

    str_length
}

#[no_mangle]
unsafe extern "C" fn stdin_reader(
    handle: *mut ::std::os::raw::c_void,
    buf: *mut ::std::os::raw::c_char,
    len: usize,
) -> isize {
    println!("reader: {:?} {:?} {:?}", handle, buf, len);

    if len == 0 {
        return 0;
    }

    let stdin_buf = STDIN.lock().unwrap();
    let buffer: &mut [u8] = slice::from_raw_parts_mut(buf as *mut u8, len as usize);

    std::ptr::copy_nonoverlapping(stdin_buf.as_ptr(), buffer.as_mut_ptr(), stdin_buf.len());

    stdin_buf.len() as isize
}

#[no_mangle]
unsafe extern "C" fn stdin_fsizer(_handle: *mut ::std::os::raw::c_void) -> usize {
    let size = STDIN.lock().unwrap().len();

    println!("fsizer: {}", size);

    size
}

#[no_mangle]
unsafe extern "C" fn stdin_closer(handle: *mut ::std::os::raw::c_void) {
    println!("closer: {:?}", handle);
}
