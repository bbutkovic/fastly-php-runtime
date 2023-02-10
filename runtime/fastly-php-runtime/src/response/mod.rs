use std::sync::Mutex;

use fastly::Response as FastlyResponse;
use lazy_static::lazy_static;

lazy_static! {
    static ref RESPONSE: Mutex<Response> = Mutex::new(Response::new());
}

pub struct Response {
    response: Option<FastlyResponse>,
}

impl Response {
    fn new() -> Self {
        Self { response: None }
    }

    pub fn initialize() {
        (*RESPONSE.lock().unwrap()).response = Some(FastlyResponse::new());
    }

    pub fn get() -> FastlyResponse {
        RESPONSE.lock().unwrap().response.take().unwrap()
    }

    // fn get_current() -> &'static mut Self {
    //     &mut
    // }

    pub fn write(str: &str) {
        RESPONSE
            .lock()
            .unwrap()
            .response
            .as_mut()
            .unwrap()
            .get_body_mut()
            .write_str(str);
    }

    pub fn send_header(name: &str, value: &str) {
        RESPONSE
            .lock()
            .unwrap()
            .response
            .as_mut()
            .unwrap()
            .append_header(name, value);
    }

    // pub fn write(str: &str) {
    //     Self::get_current_response().get_body_mut().write_str(str);
    //     // // let body = self.response.as_mut().unwrap().get_body_mut();
    //     // body.write_str(str);
    // }

    // fn write(&mut self, str: &str) {
    //     let body = self.response.as_mut().unwrap().get_body_mut();
    //     body.write_str(str);
    // }
}
