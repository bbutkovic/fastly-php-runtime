use std::{io::Write, str::FromStr};

use anyhow::{bail, Error, Ok, Result};
use fastly::{
    handle::{
        BodyHandle as FastlyBodyHandle, ResponseHandle as FastlyResponseHandle,
        StreamingBodyHandle as FastlyStreamingBodyHandle,
    },
    http::{HeaderName, HeaderValue},
};

pub struct ResponseHandle {
    response: ResponseState,
    body: Body,
}

enum ResponseState {
    Uninitialized,
    Initialized(FastlyResponseHandle),
    Used,
}

impl ResponseState {
    fn new() -> Self {
        Self::Uninitialized
    }

    fn try_borrow_mut(&mut self) -> Result<&mut FastlyResponseHandle> {
        match self {
            Self::Uninitialized => {
                let res = FastlyResponseHandle::new();
                *self = Self::Initialized(res);

                self.try_borrow_mut()
            }
            Self::Initialized(ref mut res) => Ok(res),
            Self::Used => bail!("Fastly Response Handle used"),
        }
    }

    fn take(&mut self) -> Result<FastlyResponseHandle> {
        match std::mem::replace(self, Self::Used) {
            Self::Uninitialized => Ok(FastlyResponseHandle::new()),
            Self::Initialized(res) => Ok(res),
            Self::Used => bail!("fastly response handle taken"),
        }
    }
}

enum Body {
    Uninitialized,
    Streaming(FastlyStreamingBodyHandle),
    Finished,
}

impl Body {
    pub fn new() -> Self {
        Self::Uninitialized
    }

    pub fn stream(&mut self, res: &mut ResponseState, content: &[u8]) -> Result<()> {
        match self {
            Self::Finished => bail!("response already finished"),
            Self::Uninitialized => {
                let res = res.take().unwrap();

                let mut body = FastlyBodyHandle::new();

                body.write_bytes(content);

                *self = Self::Streaming(res.stream_to_client(body));

                Ok(())
            }
            Self::Streaming(body) => {
                body.write_bytes(content);

                Ok(())
            }
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        match std::mem::replace(self, Self::Finished) {
            Self::Uninitialized | Self::Finished => Ok(()),
            Self::Streaming(mut body) => body.flush().map_err(Error::from),
        }
    }
}

impl ResponseHandle {
    pub fn new() -> Self {
        Self {
            response: ResponseState::new(),
            body: Body::new(),
        }
    }

    pub fn send_header(&mut self, name: String, value: String) -> Result<()> {
        let response = self.response.try_borrow_mut().unwrap();

        let name = HeaderName::from_str(name.as_str()).unwrap();
        let value = HeaderValue::from_str(value.as_str()).unwrap();
        response.insert_header(&name, &value);

        Ok(())
    }

    pub fn stream_response(&mut self, content: &[u8]) -> Result<()> {
        self.body.stream(&mut self.response, content)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.body.flush()
    }
}

impl Drop for ResponseHandle {
    fn drop(&mut self) {
        if self.flush().is_err() {
            println!("cannot flush")
        }
    }
}
