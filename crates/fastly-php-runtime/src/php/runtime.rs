use anyhow::Result;
use bytes::Bytes;
use php_sys::{
    _zend_file_handle__bindgen_ty_1, php_execute_script, php_request_startup, zend_file_handle,
    zend_stream_type_ZEND_HANDLE_FP,
};

use crate::fastly_ce::manager::response;

use super::sapi::init_fastly_ce_sapi;

mod globals;

#[derive(Debug)]
pub struct Runtime {
    code: Option<Bytes>,
}

impl Runtime {
    pub fn new() -> Self {
        Self { code: None }
    }

    pub fn exec(&mut self) -> Result<()> {
        init_fastly_ce_sapi();

        let code = self.code.take().unwrap();

        let code_len = code.len();

        let code = code.as_ptr();

        let primary_file: zend_file_handle = zend_file_handle {
            handle: _zend_file_handle__bindgen_ty_1 {
                fp: std::ptr::null_mut(),
            },
            filename: std::ptr::null_mut(),
            opened_path: std::ptr::null_mut(),
            type_: zend_stream_type_ZEND_HANDLE_FP as u8,
            primary_script: true,
            in_list: false,
            buf: code as *mut i8,
            len: code_len,
        };

        let primary_file = Box::into_raw(Box::new(primary_file));

        #[cfg(debug_assertions)]
        println!("Executing script");

        unsafe {
            php_request_startup();
            php_execute_script(primary_file);
        }

        let mut response = response();

        response.flush();

        Ok(())
    }

    pub fn load(&mut self, code: Bytes) {
        self.code = Some(code);
    }
}
