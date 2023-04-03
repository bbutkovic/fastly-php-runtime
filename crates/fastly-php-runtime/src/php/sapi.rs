use php_sys::{
    _sapi_module_struct, _zend_module_entry, php_module_startup, php_register_variable,
    sapi_header_struct, sapi_startup,
};

use crate::{
    fastly_ce::manager::{request, response},
    util::cstr,
};

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
        read_post: Some(fastly_ce_sapi_read_post),
        read_cookies: None,
        register_server_variables: Some(fastly_ce_sapi_register_server_vars),
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

unsafe extern "C" fn fastly_ce_sapi_read_post(
    buffer: *mut ::std::os::raw::c_char,
    count_bytes: usize,
) -> usize {
    #[cfg(debug_assertions)]
    println!("Fastly C@E read_post");

    let mut request = request();

    let buf: &mut [u8] = std::slice::from_raw_parts_mut(buffer as *mut u8, count_bytes);

    request
        .read_body_chunk(buf)
        .expect("could not read body chunk")
}

unsafe extern "C" fn fastly_ce_sapi_register_server_vars(
    track_vars_array: *mut php_sys::_zval_struct,
) {
    #[cfg(debug_assertions)]
    println!("Fastly C@E register_server_variables");
    let mut request = request();

    if let Some(remote_addr) = request.remote_address() {
        let remote_addr = remote_addr.to_string();
        register_php_variable("REMOTE_ADDR", remote_addr.as_str(), track_vars_array);
    }

    // todo: this breaks PHP
    // let server_software = format!(
    //     "Fastly Compute@Edge/{}",
    //     String::from_utf8(PHP_VERSION.to_vec()).unwrap()
    // );

    // register_php_variable(
    //     "SERVER_SOFTWARE",
    //     server_software.as_str(),
    //     track_vars_array,
    // );

    if let Some(http_version) = request.http_version() {
        let http_version = format!("HTTP/{}", http_version);
        register_php_variable("SERVER_PROTOCOL", http_version.as_str(), track_vars_array);
    }

    let request_method = request.request_method();
    register_php_variable("REQUEST_METHOD", request_method.as_str(), track_vars_array);

    if let Some(request_uri) = request.request_uri() {
        register_php_variable("REQUEST_URI", request_uri.as_str(), track_vars_array);
    }

    for (name, value) in request.headers() {
        let name = name.to_string();
        let value = value.to_string();

        let name = format!("HTTP_{}", name.replace('-', "_").to_uppercase());

        register_php_variable(name.as_str(), value.as_str(), track_vars_array);
    }
}

pub fn register_php_variable(
    variable: &str,
    value: &str,
    track_vars_array: *mut php_sys::_zval_struct,
) {
    let variable = std::ffi::CString::new(variable).unwrap();
    let value = std::ffi::CString::new(value).unwrap();

    let var = variable.as_ptr() as *const ::std::os::raw::c_char;
    let val = value.as_ptr() as *const ::std::os::raw::c_char;

    unsafe { php_register_variable(var, val, track_vars_array) };
}

unsafe extern "C" fn fastly_ce_sapi_send_header(
    sapi_header: *mut sapi_header_struct,
    _server_context: *mut ::std::os::raw::c_void,
) {
    let header = std::ffi::CStr::from_ptr((*sapi_header).header)
        .to_str()
        .unwrap();

    if let Some((name, value)) = header
        .split_once(": ")
        .map(|(name, value)| (name.to_string(), value.to_string()))
    {
        let mut response = response();

        response.send_header(name, value).unwrap();
    }
}

unsafe extern "C" fn fastly_ce_sapi_startup(
    php_sapi_module: *mut _sapi_module_struct,
) -> ::std::os::raw::c_int {
    #[cfg(debug_assertions)]
    println!("Starting up Fastly C@E SAPI");

    let ce_module_entry = fastly_ce_module::get_module();

    let startup_res = php_module_startup(
        php_sapi_module,
        &mut *(ce_module_entry as *mut ext_php_rs::ffi::_zend_module_entry
            as *mut _zend_module_entry),
        1,
    );

    #[cfg(debug_assertions)]
    println!("Fastly C@E SAPI startup result: {}", startup_res);

    startup_res
}

#[no_mangle]
unsafe extern "C" fn fastly_ce_sapi_ub_write(
    str: *const ::std::os::raw::c_char,
    str_length: usize,
) -> usize {
    #[cfg(debug_assertions)]
    println!("Fastly C@E ub_write (len {}): {:p}", str_length, str);
    let ub_write_bytes = std::ffi::CStr::from_ptr(str).to_bytes();

    let mut response = response();

    response.stream_response(ub_write_bytes).unwrap();

    str_length
}
