use std::{
    cell::RefCell,
    io::{stdin, Read},
};

use bytes::{buf::Reader, Buf, Bytes};
use php_sys::{
    _zend_file_handle__bindgen_ty_1, zend_compile_file, zend_file_handle, zend_op_array,
    zend_stream, zend_stream_type_ZEND_HANDLE_STREAM, zend_string_init_interned,
};

use crate::util::cstr;

struct ReaderHandle(Option<Reader<Bytes>>, usize);

impl Default for ReaderHandle {
    fn default() -> Self {
        Self(None, 0)
    }
}

impl ReaderHandle {
    fn init(&mut self, buffer: Bytes) {
        let size = buffer.len();
        let reader = buffer.reader();

        self.0 = Some(reader);
        self.1 = size;
    }

    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let mut reader = self.0.take().unwrap();

        let result = reader.read(buf);

        self.0 = Some(reader);

        result.map_err(anyhow::Error::from)
    }

    fn size(&self) -> usize {
        self.1
    }
}

thread_local! {
    static STDIN_READER_HANDLE: RefCell<ReaderHandle> = RefCell::new(ReaderHandle::default());
}

fn initialize_stdin_reader_handle() {
    STDIN_READER_HANDLE.with(|reader_handle| {
        let input: Bytes = stdin().bytes().map(|b| b.unwrap()).collect();
        (*reader_handle.borrow_mut()).init(input);
    });
}

fn read_stdin_into_buffer(buf: &mut [u8]) -> anyhow::Result<usize> {
    STDIN_READER_HANDLE
        .with(|reader_handle| (*reader_handle.borrow_mut()).read(buf))
        .map_err(anyhow::Error::from)
}

fn get_stdin_size() -> usize {
    STDIN_READER_HANDLE.with(|reader_handle| (*reader_handle.borrow()).size())
}

pub fn compile_from_stdin() -> *mut zend_op_array {
    initialize_stdin_reader_handle();

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

    #[cfg(debug_assertions)]
    println!("PHP compilation finished: {:?}", op_array);

    op_array
}

#[no_mangle]
unsafe extern "C" fn stdin_reader(
    handle: *mut ::std::os::raw::c_void,
    buf: *mut ::std::os::raw::c_char,
    len: usize,
) -> isize {
    #[cfg(debug_assertions)]
    println!(
        "Fastly C@E compilation stdin_reader start: {:?} {:?} {}",
        handle, buf, len
    );

    if len == 0 {
        #[cfg(debug_assertions)]
        println!("Fastly C@E compilation stdin_reader len of 0");
        return 0;
    }

    let buffer: &mut [u8] = std::slice::from_raw_parts_mut(buf as *mut u8, len as usize);

    let result = read_stdin_into_buffer(buffer).expect("could not read into buffer");

    #[cfg(debug_assertions)]
    println!("Fastly C@E compilation stdin_reader end: {}", result);

    result as isize
}

#[no_mangle]
unsafe extern "C" fn stdin_fsizer(handle: *mut ::std::os::raw::c_void) -> usize {
    #[cfg(debug_assertions)]
    println!("Fastly C@E compilation stdin_fsizer start: {:?}", handle);

    let size = get_stdin_size();

    #[cfg(debug_assertions)]
    println!("Fastly C@E compilation stdin_fsizer end: {}", size);

    size
}

#[no_mangle]
unsafe extern "C" fn stdin_closer(handle: *mut ::std::os::raw::c_void) {
    #[cfg(debug_assertions)]
    println!("Fastly C@E compilation stdin_closer: {:?}", handle);
}
