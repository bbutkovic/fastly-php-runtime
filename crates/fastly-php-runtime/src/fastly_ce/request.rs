use std::{
    io::{Cursor, Read},
    net::IpAddr,
};

use anyhow::{bail, Result};
use fastly::{handle::client_ip_addr, Request as FastlyRequest};

pub struct RequestHandle {
    request: RequestState,
    body: Body,
}

enum RequestState {
    Uninitialized,
    Initialized(FastlyRequest),
    Used,
}

enum Body {
    Uninitialized,
    Reading(Cursor<Vec<u8>>),
    // todo?
    Finished,
}

impl Body {
    pub fn new() -> Self {
        Self::Uninitialized
    }

    pub fn read(&mut self, req: &mut RequestState, buffer: &mut [u8]) -> Result<usize> {
        match self {
            Self::Finished => bail!("response already finished"),
            Self::Uninitialized => {
                let mut req = req.take().unwrap();

                let mut cursor = Cursor::new(req.take_body_bytes());

                let read = cursor.read(buffer)?;

                *self = Self::Reading(cursor);

                Ok(read)
            }
            Self::Reading(cursor) => Ok(cursor.read(buffer)?),
        }
    }
}

impl RequestState {
    fn new() -> Self {
        Self::Uninitialized
    }

    fn try_borrow_mut(&mut self) -> Result<&mut FastlyRequest> {
        match self {
            Self::Uninitialized => {
                let res = FastlyRequest::from_client();
                *self = Self::Initialized(res);

                self.try_borrow_mut()
            }
            Self::Initialized(ref mut res) => Ok(res),
            Self::Used => bail!("fastly request handle taken"),
        }
    }

    fn try_borrow(&mut self) -> Result<&FastlyRequest> {
        self.try_borrow_mut().map(|req| &*req)
    }

    fn take(&mut self) -> Result<FastlyRequest> {
        match std::mem::replace(self, Self::Used) {
            Self::Uninitialized => Ok(FastlyRequest::from_client()),
            Self::Initialized(res) => Ok(res),
            Self::Used => bail!("fastly request handle taken"),
        }
    }
}

impl RequestHandle {
    pub fn new() -> Self {
        Self {
            request: RequestState::new(),
            body: Body::new(),
        }
    }

    pub fn remote_address(&mut self) -> Option<IpAddr> {
        client_ip_addr()
    }

    pub fn headers(&mut self) -> Vec<(String, String)> {
        (*self
            .request
            .try_borrow()
            .unwrap()
            .get_headers()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect::<Vec<_>>())
        .to_vec()
    }

    pub fn http_version(&mut self) -> Option<String> {
        match self.request.try_borrow().unwrap().get_version() {
            fastly::http::Version::HTTP_09 => Some("0.9"),
            fastly::http::Version::HTTP_10 => Some("1.0"),
            fastly::http::Version::HTTP_11 => Some("1.1"),
            fastly::http::Version::HTTP_2 => Some("2.0"),
            fastly::http::Version::HTTP_3 => Some("3.0"),
            _ => None,
        }
        .map(|v| v.to_string())
    }

    pub fn request_uri(&mut self) -> Option<String> {
        Some(self.request.try_borrow().unwrap().get_url().to_string())
    }

    pub fn request_method(&mut self) -> String {
        self.request
            .try_borrow()
            .unwrap()
            .get_method_str()
            .to_string()
    }

    // todo
    // pub fn query_params(&mut self) -> Option<Vec<(String, String)>> {
    //     match &mut self.state {
    //         RequestState::Uninitialized => self.initialize_request().query_params(),
    //         RequestState::Request(req) => {
    //             let req = req.take().unwrap();

    //             let query_params = req.get_query::<Vec<(String, String)>>();

    //             self.state = RequestState::Request(Some(req));

    //             query_params.ok()
    //         }
    //         _ => unreachable!(),
    //     }
    // }

    pub fn query_string(&mut self) -> Option<String> {
        self.request
            .try_borrow()
            .unwrap()
            .get_url()
            .query()
            .map(|s| s.to_string())
    }

    pub fn read_body_chunk(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        self.body.read(&mut self.request, buf)
    }

    // pub fn post_params(&mut self) -> Option<Vec<(String, String)>> {
    //     match &mut self.state {
    //         RequestState::Uninitialized => self.initialize_request().post_params(),
    //         RequestState::Request(req) => {
    //             let req = req.take().unwrap();

    //             let post_params = req.get_body::<Vec<(String, String)>>();

    //             self.state = RequestState::Request(Some(req));

    //             post_params.ok()
    //         }
    //     }
    // }
}
