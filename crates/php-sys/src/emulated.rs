// todo

#[no_mangle]
extern "C" fn getuid() -> i32 {
    1000
}

#[no_mangle]
extern "C" fn getgid() -> i32 {
    1000
}

#[no_mangle]
extern "C" fn getgroups(_size: i32, _list: *mut i32) -> i32 {
    0
}

#[no_mangle]
extern "C" fn chmod(_path: *const i8, _mode: i32) -> i32 {
    0
}

#[no_mangle]
extern "C" fn umask(_mask: i32) -> i32 {
    0
}