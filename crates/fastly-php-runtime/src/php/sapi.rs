use php_sys::{
    _sapi_module_struct, _zend_module_entry, php_module_startup, sapi_header_struct, sapi_startup,
};

use crate::{fastly_ce::manager::response, util::cstr};

pub fn init_fastly_ce_sapi() {
    let name = cstr!("fastly-ce");
    let pretty_name = cstr!("Fastly Compute@Edge");

    // todo: allow some form of configuration through a php.ini file?
    let ini_entries = "html_errors=0\n
	register_argc_argv=1\n
	implicit_flush=1\n
	output_buffering=0\n
	max_execution_time=0\n
	max_input_time=-1\n";

    let ini_entries = Box::into_raw(Box::new(ini_entries));

    let fastly_ce_sapi: _sapi_module_struct = _sapi_module_struct {
        name: name.as_ptr() as *mut i8,
        pretty_name: pretty_name.as_ptr() as *mut i8,
        startup: Some(fastly_ce_sapi_startup),
        shutdown: None,
        activate: None,
        deactivate: None,
        ub_write: Some(fastly_ce_sapi_ub_write),
        flush: None,
        get_stat: None,
        getenv: None,
        sapi_error: None,
        header_handler: None,
        send_headers: None,
        send_header: Some(fastly_ce_sapi_send_header),
        read_post: None,
        read_cookies: None,
        register_server_variables: None,
        log_message: None,
        get_request_time: None,
        terminate_process: None,
        php_ini_path_override: std::ptr::null_mut(),
        default_post_reader: None,
        treat_data: None,
        executable_location: std::ptr::null_mut(),
        php_ini_ignore: 0,
        php_ini_ignore_cwd: 0,
        get_fd: None,
        force_http_10: None,
        get_target_uid: None,
        get_target_gid: None,
        input_filter: None,
        ini_defaults: None,
        phpinfo_as_text: 0,
        ini_entries: ini_entries as *mut i8,
        additional_functions: std::ptr::null_mut(),
        input_filter_init: None,
    };

    let fastly_ce_sapi = Box::into_raw(Box::new(fastly_ce_sapi));

    unsafe { sapi_startup(fastly_ce_sapi) };

    unsafe { (*fastly_ce_sapi).startup.unwrap()(fastly_ce_sapi) };
}

unsafe extern "C" fn fastly_ce_sapi_send_header(
    sapi_header: *mut sapi_header_struct,
    _server_context: *mut ::std::os::raw::c_void,
) {
    let header = std::ffi::CStr::from_ptr((*sapi_header).header)
        .to_str()
        .unwrap();

    match header
        .split_once(": ")
        .map(|(name, value)| (name.to_string(), value.to_string()))
    {
        Some((name, value)) => {
            let mut response = response();

            response.send_header(name, value).unwrap();
        }
        None => {
            // todo handle response code?
        }
    }
}

unsafe extern "C" fn fastly_ce_sapi_startup(
    php_sapi_module: *mut _sapi_module_struct,
) -> ::std::os::raw::c_int {
    let ce_module_entry = fastly_ce_module::get_module();

    php_module_startup(
        php_sapi_module,
        &mut *(ce_module_entry as *mut ext_php_rs::ffi::_zend_module_entry
            as *mut _zend_module_entry),
        1,
    )
}

#[no_mangle]
unsafe extern "C" fn fastly_ce_sapi_ub_write(
    str: *const ::std::os::raw::c_char,
    str_length: usize,
) -> usize {
    let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

    let mut response = response();

    response.stream_response(str.to_string()).unwrap();

    str_length
}
