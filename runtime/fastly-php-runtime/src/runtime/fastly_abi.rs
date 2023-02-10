use std::ffi::CString;

use crate::php;

use crate::util::{cstr, cstring};

use lazy_static::lazy_static;

// lazy_static! {
//     static ref PHP_FASTLY_MODULE: php::sapi_module_struct = unsafe { &php::sapi_module_struct {} };
// }

// pub const PHP_FASTLY_MODULE: &'static php::sapi_module_struct = &php::sapi_module_struct {
//     name: std::ptr::null_mut(),
//     pretty_name: std::ptr::null_mut(),
//     startup: None,
//     shutdown: None,
//     activate: None,
//     deactivate: None,
//     ub_write: None,
//     flush: None,
//     get_stat: None,
//     getenv: None,
//     sapi_error: None,
//     header_handler: None,
//     send_headers: None,
//     send_header: None,
//     read_post: None,
//     read_cookies: None,
//     register_server_variables: None,
//     log_message: None,
//     get_request_time: None,
//     terminate_process: None,
//     php_ini_path_override: std::ptr::null_mut(),
//     default_post_reader: None,
//     treat_data: None,
//     executable_location: std::ptr::null_mut(),
//     php_ini_ignore: 0,
//     php_ini_ignore_cwd: 0,
//     get_fd: None,
//     force_http_10: None,
//     get_target_uid: None,
//     get_target_gid: None,
//     input_filter: None,
//     ini_defaults: None,
//     phpinfo_as_text: 0,
//     ini_entries: std::ptr::null_mut(),
//     additional_functions: std::ptr::null(),
//     input_filter_init: None,
// };

// TODO set expected resources
// php::tsrm_startup(threads, 1, 0, ptr::null_mut());
// php::ts_resource_ex(0, ptr::null_mut());
// zend_tsrmls_cache_update();

// php::zend_signal_startup();

// let mut module = Box::new(php::sapi_module_struct::default());
// let name = CString::new("php-rpm").unwrap();
// let pretty_name = CString::new("PHP Rust Process Manager").unwrap();

// module.name = name.into_raw();
// module.pretty_name = pretty_name.into_raw();
// module.startup = Some(sapi_server_startup);
// module.shutdown = Some(sapi_server_shutdown);
// module.ub_write = Some(sapi_server_ub_write);
// module.flush = Some(sapi_server_flush);
// module.sapi_error = Some(php::zend_error);
// module.send_headers = Some(sapi_server_send_headers);
// module.read_post = Some(sapi_server_read_post);
// module.read_cookies = Some(sapi_server_read_cookies);
// module.register_server_variables = Some(sapi_server_register_variables);
// module.log_message = Some(sapi_server_log_message);

// let module_ptr = Box::into_raw(module);

// // TODO error check
// // this function assigns the module pointer to the `php::sapi_module` global variable
// php::sapi_startup(module_ptr);

// let request_method = CString::new("GET").unwrap();
// let path_translated = CString::new("/home/herman/projects/php-rpm/tests/index.php").unwrap();
// let content_type = CString::new("text/html").unwrap();
// (*sg_request_info()).request_method = request_method.as_ptr();
// (*sg_request_info()).content_length = 0;
// (*sg_request_info()).path_translated = path_translated.into_raw();
// (*sg_request_info()).content_type = content_type.as_ptr();

// // this function also assigns the module pointer to the `php::sapi_module` global variable
// php::php_module_startup(module_ptr, ptr::null_mut(), 0);
