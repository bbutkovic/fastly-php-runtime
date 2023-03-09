use std::{io::Write, str::FromStr};

use fastly::{
    handle::{
        BodyHandle as FastlyBodyHandle, ResponseHandle as FastlyResponseHandle,
        StreamingBodyHandle as FastlyStreamingBodyHandle,
    },
    http::{HeaderName, HeaderValue},
};

pub struct ResponseHandle {
    state: ResponseState,
}

enum ResponseState {
    Uninitialized,
    Response(Option<FastlyResponseHandle>),
    StreamingBodyResponse(Option<FastlyStreamingBodyHandle>),
    Finished,
}

impl ResponseHandle {
    pub fn new() -> Self {
        Self {
            state: ResponseState::Uninitialized,
        }
    }

    fn initialize_response<'a>(&'a mut self) -> &'a mut Self {
        self.state = ResponseState::Response(Some(FastlyResponseHandle::new()));
        self
    }

    fn initialized_response<'a>(&'a mut self) -> &'a mut Self {
        match &self.state {
            ResponseState::Uninitialized => self.initialize_response(),
            _ => self,
        }
    }

    pub fn send_header<'a>(&'a mut self, name: String, value: String) -> Result<&'a mut Self, ()> {
        match &mut self.state {
            ResponseState::Uninitialized => self.initialize_response().send_header(name, value),
            ResponseState::Response(res) => {
                let name = HeaderName::from_str(name.as_str()).unwrap();
                let value = HeaderValue::from_str(value.as_str()).unwrap();

                let mut res = res.take().unwrap();
                res.append_header(&name, &value);

                self.state = ResponseState::Response(Some(res));

                Ok(self)
            }
            ResponseState::StreamingBodyResponse(_) => {
                // todo: change this to error
                panic!("response body already started streaming")
            }
            ResponseState::Finished => panic!("response finished"),
        }
    }

    pub fn stream_response<'a>(&'a mut self, content: String) -> Result<&'a mut Self, ()> {
        match &mut self.state {
            ResponseState::Uninitialized => self.initialize_response().stream_response(content),
            ResponseState::Response(res) => {
                let mut body = FastlyBodyHandle::new();
                body.write_str(content.as_str());

                let res = res.take().unwrap();
                let streaming_body = res.stream_to_client(body);

                self.state = ResponseState::StreamingBodyResponse(Some(streaming_body));

                Ok(self)
            }
            ResponseState::StreamingBodyResponse(streaming_body) => {
                let mut streaming_body = streaming_body.take().unwrap();

                streaming_body.write_str(content.as_str());

                self.state = ResponseState::StreamingBodyResponse(Some(streaming_body));

                Ok(self)
            }
            ResponseState::Finished => {
                panic!("response finished already");
            }
        }
    }

    // todo: clean this up
    pub fn flush<'a>(&'a mut self) {
        match &mut self.state {
            ResponseState::Uninitialized => {
                panic!("response not initialized")
            }
            ResponseState::Response(_) => todo!(),
            ResponseState::StreamingBodyResponse(streaming_body) => {
                let mut streaming_body = streaming_body.take().unwrap();

                streaming_body.flush().unwrap();
            }
            ResponseState::Finished => todo!(),
        }
    }
}
