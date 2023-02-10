pub(crate) mod php;

// use std::{
//     borrow::BorrowMut,
//     ffi::{CStr, CString},
// };

// use fastly::Request;

// use crate::{php, response::Response};

// use lazy_static::lazy_static;

// mod fastly_abi;

// const ABI_FUNCTIONS: &'static [php::_zend_function_entry; 1] = &[php::_zend_function_entry {
//     fname: cstr!("fastly_php_geo_lookup").as_ptr(),
//     handler: Some(fastly_php_geo_lookup),
//     arg_info: [php::_zend_internal_arg_info {
//         name: cstr!("ip").as_ptr(),
//         type_: 1,
//         pass_by_reference: 0,
//         is_variadic: 0,
//     }]
//     .as_ptr(),
//     num_args: 1,
//     flags: 0,
// }];

// unsafe extern "C" fn fastly_php_geo_lookup(
//     execute_data: *mut php::zend_execute_data,
//     return_value: *mut php::zval,
// ) {
// }

/*
EMBED_SAPI_API int php_embed_init(int argc, char **argv)
{
#if defined(SIGPIPE) && defined(SIG_IGN)
    signal(SIGPIPE, SIG_IGN); /* ignore SIGPIPE in standalone mode so
                                 that sockets created via fsockopen()
                                 don't kill PHP if the remote site
                                 closes it.  in apache|apxs mode apache
                                 does that for us!  thies@thieso.net
                                 20000419 */
#endif

#ifdef ZTS
    php_tsrm_startup();
# ifdef PHP_WIN32
    ZEND_TSRMLS_CACHE_UPDATE();
# endif
#endif

    zend_signal_startup();

    /* SAPI initialization (SINIT)
     *
     * Initialize the SAPI globals (memset to 0). After this point we can set
     * SAPI globals via the SG() macro.
     *
     * Reentrancy startup.
     *
     * This also sets 'php_embed_module.ini_entries = NULL' so we cannot
     * allocate the INI entries until after this call.
     */
    sapi_startup(&php_embed_module);

#ifdef PHP_WIN32
    _fmode = _O_BINARY;			/*sets default for file streams to binary */
    setmode(_fileno(stdin), O_BINARY);		/* make the stdio mode be binary */
    setmode(_fileno(stdout), O_BINARY);		/* make the stdio mode be binary */
    setmode(_fileno(stderr), O_BINARY);		/* make the stdio mode be binary */
#endif

    /* This hard-coded string of INI settings is parsed and read into PHP's
     * configuration hash table at the very end of php_init_config(). This
     * means these settings will overwrite any INI settings that were set from
     * an INI file.
     *
     * To provide overwritable INI defaults, hook the ini_defaults function
     * pointer that is part of the sapi_module_struct
     * (php_embed_module.ini_defaults).
     *
     *     void (*ini_defaults)(HashTable *configuration_hash);
     *
     * This callback is invoked as soon as the configuration hash table is
     * allocated so any INI settings added via this callback will have the
     * lowest precedence and will allow INI files to overwrite them.
     */
    php_embed_module.ini_entries = HARDCODED_INI;

    /* SAPI-provided functions. */
    php_embed_module.additional_functions = additional_functions;

    if (argv) {
        php_embed_module.executable_location = argv[0];
    }

    /* Module initialization (MINIT) */
    if (php_embed_module.startup(&php_embed_module) == FAILURE) {
        return FAILURE;
    }

    /* Do not chdir to the script's directory. This is akin to calling the CGI
     * SAPI with '-C'.
     */
    SG(options) |= SAPI_OPTION_NO_CHDIR;

    SG(request_info).argc=argc;
    SG(request_info).argv=argv;

    /* Request initialization (RINIT) */
    if (php_request_startup() == FAILURE) {
        php_module_shutdown();
        return FAILURE;
    }

    SG(headers_sent) = 1;
    SG(request_info).no_headers = 1;
    php_register_variable("PHP_SELF", "-", NULL);

    return SUCCESS;
}

*/

// const PHP_FASTLY_MODULE: &'static php::sapi_module_struct = &php::sapi_module_struct {
//     name: cstr!("fastly").borrow_mut().as_ptr(),
//     pretty_name: todo!(),
//     startup: todo!(),
//     shutdown: todo!(),
//     activate: todo!(),
//     deactivate: todo!(),
//     ub_write: todo!(),
//     flush: todo!(),
//     get_stat: todo!(),
//     getenv: todo!(),
//     sapi_error: todo!(),
//     header_handler: todo!(),
//     send_headers: todo!(),
//     send_header: todo!(),
//     read_post: todo!(),
//     read_cookies: todo!(),
//     register_server_variables: todo!(),
//     log_message: todo!(),
//     get_request_time: todo!(),
//     terminate_process: todo!(),
//     php_ini_path_override: todo!(),
//     default_post_reader: todo!(),
//     treat_data: todo!(),
//     executable_location: todo!(),
//     php_ini_ignore: todo!(),
//     php_ini_ignore_cwd: todo!(),
//     get_fd: todo!(),
//     force_http_10: todo!(),
//     get_target_uid: todo!(),
//     get_target_gid: todo!(),
//     input_filter: todo!(),
//     ini_defaults: todo!(),
//     phpinfo_as_text: todo!(),
//     ini_entries: todo!(),
//     additional_functions: todo!(),
//     input_filter_init: todo!(),
// };

// lazy_static::lazy_static! {
//     static ref PHP_FASTLY_MODULE: php::sapi_module_struct = {

//     };

//     // static ref PHP: Mutex<Php> = Mutex::new(Php::new());
// }

// fn

// pub fn execute(req: Request, code: &str) -> Option<String> {
//     let code = CString::new(code).unwrap();

//     unsafe {
//         php::php_embed_module.ub_write = Some(embed_write);
//         php::php_embed_module.send_header = Some(embed_send_header);
//         php::php_embed_module.read_post = Some(embed_read_post);

//         php::php_embed_init(0, std::ptr::null_mut());
//         // php::php_embed_module.additional_functions = ABI_FUNCTIONS.as_ptr();

//         let script = std::ptr::null_mut::<php::zval>();

//         let code_raw = code.into_raw();

//         php::zend_eval_string(
//             code_raw,
//             script,
//             CString::new("fastly-php").unwrap().into_raw(),
//         );

//         php::_convert_to_string(script);

//         println!("return: {:?}", (*(*script).value.str).val);
//     }

//     None
// }

// unsafe extern "C" fn embed_write(
//     str: *const ::std::os::raw::c_char,
//     str_length: php::size_t,
// ) -> php::size_t {
//     let str = std::ffi::CStr::from_ptr(str).to_str().unwrap();

//     Response::write(str);

//     str_length
// }

// unsafe extern "C" fn embed_send_header(
//     sapi_header: *mut php::sapi_header_struct,
//     server_context: *mut ::std::os::raw::c_void,
// ) {
//     let header = std::ffi::CStr::from_ptr((*sapi_header).header)
//         .to_str()
//         .unwrap();

//     let (name, value) = header.split_once(":").unwrap();

//     println!("header: {} {}", name, value);

//     Response::send_header(name, value);
// }

// unsafe extern "C" fn embed_read_post(
//     buffer: *mut ::std::os::raw::c_char,
//     count_bytes: php::size_t,
// ) -> php::size_t {
//     todo!()
// }

// fn setup_additional_functions() -> Vec<php::zend_function_entry> {
//     let mut functions = Vec::new();

//     functions.push(php::zend_function_entry {
//         fname: CString::new("fastly_php_geo_lookup").unwrap().into_raw(),
//         handler: Some(unsafe { std::mem::transmute(getpid as usize) }),
//         arg_info: std::ptr::null_mut(),
//         num_args: 0,
//         flags: 0,
//     });

//     // functions.push(php::zend_function_entry {
//     //     fname: CString::new("times").unwrap().into_raw(),
//     //     handler: Some(unsafe { std::mem::transmute(times as usize) }),
//     //     arg_info: std::ptr::null_mut(),
//     //     num_args: 0,
//     //     flags: 0,
//     // });

//     // functions.push(php::zend_function_entry {
//     //     fname: std::ptr::null_mut(),
//     //     handler: None,
//     //     arg_info: std::ptr::null_mut(),
//     //     num_args: 0,
//     //     flags: 0,
//     // });

//     functions
// }

// pub fn execute() {
//     unsafe {
//         let a = php::phar_globals.phar_alias_map.arData.;
//     }

//     todo!()
// }
