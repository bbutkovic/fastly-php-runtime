use std::str::FromStr;

use fastly::{
    handle::{BodyHandle, ResponseHandle, StreamingBodyHandle},
    http::{HeaderName, HeaderValue},
};

pub struct FastlyResponseHandle {
    state: FastlyResponseState,
}

enum FastlyResponseState {
    Uninitialized,
    Response(Option<ResponseHandle>),
    StreamingBodyResponse(Option<StreamingBodyHandle>),
    Finished,
}

impl FastlyResponseHandle {
    pub fn new() -> Self {
        Self {
            state: FastlyResponseState::Uninitialized,
        }
    }

    fn initialize_response<'a>(&'a mut self) -> &'a mut Self {
        self.state = FastlyResponseState::Response(Some(ResponseHandle::new()));
        self
    }

    pub fn send_header<'a>(
        &'a mut self,
        name: String,
        value: String,
    ) -> Result<&'a mut FastlyResponseHandle, ()> {
        match &mut self.state {
            FastlyResponseState::Uninitialized => {
                self.initialize_response().send_header(name, value)
            }
            FastlyResponseState::Response(res) => {
                let name = HeaderName::from_str(name.as_str()).unwrap();
                let value = HeaderValue::from_str(value.as_str()).unwrap();

                res.take().unwrap().append_header(&name, &value);

                Ok(self)
            }
            FastlyResponseState::StreamingBodyResponse(_) => {
                // todo: change this to error
                panic!("response body already started streaming")
            }
            FastlyResponseState::Finished => panic!("response finished"),
        }
    }

    pub fn stream_response<'a>(
        &'a mut self,
        content: String,
    ) -> Result<&'a mut FastlyResponseHandle, ()> {
        match &mut self.state {
            FastlyResponseState::Uninitialized => {
                self.initialize_response().stream_response(content)
            }
            FastlyResponseState::Response(res) => {
                let mut body = BodyHandle::new();
                body.write_str(content.as_str());

                let res = res.take().unwrap();
                let streaming_body = res.stream_to_client(body);

                self.state = FastlyResponseState::StreamingBodyResponse(Some(streaming_body));

                Ok(self)
            }
            FastlyResponseState::StreamingBodyResponse(streaming_body) => {
                streaming_body.take().unwrap().write_str(content.as_str());

                Ok(self)
            }
            FastlyResponseState::Finished => {
                panic!("response finished already");
            }
        }
    }
}
