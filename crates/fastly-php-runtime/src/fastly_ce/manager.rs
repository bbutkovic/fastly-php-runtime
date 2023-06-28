use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;

lazy_static! {
    static ref REQUEST_HANDLE: Mutex<RequestHandle> = Mutex::new(RequestHandle::new());
    static ref RESPONSE_HANDLE: Mutex<ResponseHandle> = Mutex::new(ResponseHandle::new());
}

use super::{request::RequestHandle, response::ResponseHandle};

pub fn request() -> MutexGuard<'static, RequestHandle> {
    REQUEST_HANDLE.lock().unwrap()
}

pub fn response() -> MutexGuard<'static, ResponseHandle> {
    RESPONSE_HANDLE.lock().unwrap()
}
