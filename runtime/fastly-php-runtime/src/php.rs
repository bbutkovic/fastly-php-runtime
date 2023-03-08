use core::slice;
use std::{
    io::{self, Read},
    ptr::{null_mut, NonNull},
    str::FromStr,
    sync::Mutex,
};

use bytes::Bytes;
use ext_php_rs;

use fastly_ce_module;
use lazy_static::lazy_static;
use php_sys::*;

use crate::{
    fastly_ce::{request::FastlyRequestHandle, response::FastlyResponseHandle},
    util::cstr,
};

lazy_static! {
    // todo: stdin is unnecessary, consume directly
    static ref STDIN: Mutex<Bytes> = Mutex::new(Bytes::new());
    static ref REQ_HANDLE: Mutex<FastlyRequestHandle> = Mutex::new(FastlyRequestHandle::new());
    static ref RES_HANDLE: Mutex<FastlyResponseHandle> = Mutex::new(FastlyResponseHandle::new());
}

pub fn execute_compiled_with_ce(op_array: *mut zend_op_array) {
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
        php_embed_module.send_header = Some(embed_send_header);
        php_embed_init(0, null_mut());
    }
}

unsafe extern "C" fn embed_send_header(
    sapi_header: *mut sapi_header_struct,
    _server_context: *mut ::std::os::raw::c_void,
) {
    let mut res_body_handle_global = RES_HANDLE.lock().unwrap();

    let header = std::ffi::CStr::from_ptr((*sapi_header).header)
        .to_str()
        .unwrap();

    let (name, value) = header
        .split_once(":")
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .unwrap();

    println!("HEADER OUT: {}: {}", name, value);

    res_body_handle_global.send_header(name, value).unwrap();

    todo!()
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
    (*STDIN.lock().unwrap()) = io::stdin().bytes().map(|b| b.unwrap()).collect();

    let compile_file = unsafe { zend_compile_file.unwrap() };
    let init_string = unsafe { zend_string_init_interned.unwrap() };

    let filename = cstr!("index.php");
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

    let mut res_body_handle_global = RES_HANDLE.lock().unwrap();

    res_body_handle_global
        .stream_response(str.to_string())
        .unwrap();

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
