use std::{io::Cursor, ptr, slice, sync::Mutex};

use bytes::Bytes;

use lazy_static::lazy_static;

lazy_static! {
    static ref PHAR: Mutex<Bytes> = Mutex::new(Bytes::new());
}

use crate::runtime::php;

pub mod phar {
    use std::{ffi::CString, io::Cursor};

    use crate::archive::embed_write;

    use super::{embed_phar_closer, embed_phar_fsizer, embed_phar_reader};

    use {
        super::PHAR,
        crate::runtime::php,
        bytes::Bytes,
        std::io::{self, Read},
    };

    pub fn execute_loaded() {
        unsafe {
            php::php_embed_module.ub_write = Some(embed_write);

            php::php_embed_init(0, std::ptr::null_mut());
        }

        unsafe {
            let primary_file: php::zend_file_handle = php::zend_file_handle {
                filename: crate::util::cstr!("test.php").as_ptr(),
                type_: php::zend_stream_type_ZEND_HANDLE_STREAM,
                opened_path: std::ptr::null_mut(),
                free_filename: 0,
                buf: std::ptr::null_mut(),
                len: 0,
                handle: php::_zend_file_handle__bindgen_ty_1 {
                    stream: php::zend_stream {
                        reader: Some(embed_phar_reader),
                        fsizer: Some(embed_phar_fsizer),
                        closer: Some(embed_phar_closer),
                        isatty: 0,
                        handle: std::ptr::null_mut(),
                    },
                },
            };

            let primary = Box::new(primary_file);

            println!("executing primary file:");
            let ret = php::zend_execute_scripts(8, std::ptr::null_mut(), 1, primary);
            println!("finished: {}", ret);
        }
    }

    pub fn store_from_stdin() {
        store_from_bytes(
            io::stdin()
                .bytes()
                .into_iter()
                .map(|b| b.unwrap())
                .collect(),
        );
    }

    pub fn store_from_bytes(bytes: Bytes) {
        (*PHAR.lock().unwrap()) = bytes;
    }
}

unsafe extern "C" fn embed_phar_reader(
    handle: *mut ::std::os::raw::c_void,
    buf: *mut ::std::os::raw::c_char,
    len: php::size_t,
) -> php::ssize_t {
    println!("reader: {:?} {:?} {:?}", handle, buf, len);
    let mut phar = PHAR.lock().unwrap();

    if phar.is_empty() || buf.is_null() {
        return -1;
    }

    let buffer: &mut [u8] = slice::from_raw_parts_mut(buf as *mut u8, len as usize);

    if buffer.len() < phar.len() {
        return -1;
    }

    ptr::copy_nonoverlapping(phar.as_ptr(), buffer.as_mut_ptr(), phar.len());

    phar.len() as php::ssize_t
}

unsafe extern "C" fn embed_phar_fsizer(_handle: *mut ::std::os::raw::c_void) -> php::size_t {
    let len = (*PHAR.lock().unwrap()).len() - 1;
    println!("fsizer: {:?}", len);
    return len as php::size_t;
}

unsafe extern "C" fn embed_phar_closer(handle: *mut ::std::os::raw::c_void) {}

unsafe extern "C" fn embed_write(
    str: *const ::std::os::raw::c_char,
    str_length: php::size_t,
) -> php::size_t {
    let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

    println!("a: {}", str);

    str_length
}
