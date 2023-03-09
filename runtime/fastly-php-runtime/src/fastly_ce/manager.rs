use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;

lazy_static! {
    static ref REQ_HANDLE: Mutex<RequestHandle> = Mutex::new(RequestHandle::new());
    static ref RES_HANDLE: Mutex<ResponseHandle> = Mutex::new(ResponseHandle::new());
}

use super::{request::RequestHandle, response::ResponseHandle};

pub fn request() -> MutexGuard<'static, RequestHandle> {
    REQ_HANDLE.lock().unwrap()
}

pub fn response() -> MutexGuard<'static, ResponseHandle> {
    RES_HANDLE.lock().unwrap()
}
