use std::{io::Read, sync::Mutex};

use bytes::Bytes;
use lazy_static::lazy_static;
use php_sys::{
    _zend_file_handle__bindgen_ty_1, zend_compile_file, zend_file_handle, zend_op_array,
    zend_stream, zend_stream_type_ZEND_HANDLE_STREAM, zend_string_init_interned,
};

use crate::util::cstr;

lazy_static! {
    // todo: stdin is unnecessary, consume directly
    static ref STDIN: Mutex<Bytes> = Mutex::new(Bytes::new());
}

pub fn compile_from_stdin() -> *mut zend_op_array {
    (*STDIN.lock().unwrap()) = std::io::stdin().bytes().map(|b| b.unwrap()).collect();

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
unsafe extern "C" fn stdin_reader(
    _handle: *mut ::std::os::raw::c_void,
    buf: *mut ::std::os::raw::c_char,
    len: usize,
) -> isize {
    if len == 0 {
        return 0;
    }

    let stdin_buf = STDIN.lock().unwrap();
    let buffer: &mut [u8] = std::slice::from_raw_parts_mut(buf as *mut u8, len as usize);

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
