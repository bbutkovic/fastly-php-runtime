use core::slice;
use std::{
    io::{self, Read},
    ptr::{null_mut, NonNull},
    sync::Mutex,
};

use bytes::Bytes;
use ext_php_rs;
use fastly::handle::{BodyHandle, RequestHandle, ResponseHandle, StreamingBodyHandle};
use fastly_ce_module;
use lazy_static::lazy_static;
use php_sys::*;

use crate::util::cstr;

lazy_static! {
    // todo: stdin is unnecessary, consume directly
    static ref STDIN: Mutex<Bytes> = Mutex::new(Bytes::new());
    static ref RES_STREAMING_BODY_HANDLE: Mutex<Option<StreamingBodyHandle>> = Mutex::new(None);
}

pub fn execute_compiled_with_ce(
    op_array: *mut zend_op_array,
    req_handle: RequestHandle,
    req_body_handle: BodyHandle,
    res_handle: ResponseHandle,
    res_body_handle: BodyHandle,
) {
    let res_streaming_body_handle = res_handle.stream_to_client(res_body_handle);

    (*RES_STREAMING_BODY_HANDLE.lock().unwrap()) = Some(res_streaming_body_handle);

    execute_compiled(op_array);
}

pub fn execute_compiled(op_array: *mut zend_op_array) {
    println!("Started executing PHP");
    unsafe { zend_execute(op_array, null_mut()) };
    log_exceptions();

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
    unsafe {
        php_embed_module.startup = Some(php_embed_startup);
        php_embed_module.ub_write = Some(embed_write);
        php_embed_init(0, null_mut());
    }
}

// workaround for loading the fastly-ce module, todo: implement our own sapi
unsafe extern "C" fn php_embed_startup(
    php_sapi_module: *mut _sapi_module_struct,
) -> ::std::os::raw::c_int {
    let ce_module_entry = fastly_ce_module::get_module();

    php_module_startup(php_sapi_module, convert(ce_module_entry), 1);

    0 as ::std::os::raw::c_int
}

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

#[no_mangle]
unsafe extern "C" fn embed_write(str: *const ::std::os::raw::c_char, str_length: usize) -> usize {
    let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

    let mut res_body_handle_global = RES_STREAMING_BODY_HANDLE.lock().unwrap();

    if let Some(mut res_body_handle) = res_body_handle_global.take() {
        res_body_handle.write_str(str);
        *res_body_handle_global = Some(res_body_handle);
    }

    str_length
}

#[no_mangle]
unsafe extern "C" fn stdin_reader(
    _handle: *mut ::std::os::raw::c_void,
    buf: *mut ::std::os::raw::c_char,
    len: usize,
) -> isize {
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

    size
}

#[no_mangle]
unsafe extern "C" fn stdin_closer(handle: *mut ::std::os::raw::c_void) {
    println!("closer: {:?}", handle);
}
