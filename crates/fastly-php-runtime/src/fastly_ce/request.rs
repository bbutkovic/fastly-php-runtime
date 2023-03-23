use std::{
    io::{Cursor, Read},
    net::IpAddr,
};

use fastly::{
    handle::{client_ip_addr, RequestHandle as FastlyRequestHandle},
    http::{HeaderName, HeaderValue},
    Body, Request,
};
use url::Position;

pub struct RequestHandle {
    state: RequestState,
}

enum RequestState {
    Uninitialized,
    Request(Option<Request>),
    ReadingBody(Option<Cursor<Vec<u8>>>),
}

impl RequestHandle {
    pub fn new() -> Self {
        Self {
            state: RequestState::Uninitialized,
        }
    }

    fn initialize_request<'a>(&'a mut self) -> &'a mut Self {
        self.state = RequestState::Request(Some(Request::from_client()));
        self
    }

    pub fn remote_address(&mut self) -> Option<IpAddr> {
        client_ip_addr()
    }

    pub fn headers(&mut self) -> Vec<(String, String)> {
        match &mut self.state {
            RequestState::Uninitialized => self.initialize_request().headers(),
            RequestState::Request(req) => {
                let req = req.take().unwrap();

                let headers: Vec<(String, String)> = req
                    .get_headers()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
                    .collect();

                self.state = RequestState::Request(Some(req));

                headers
            }
            _ => unreachable!(),
        }
    }

    pub fn http_version(&mut self) -> Option<String> {
        match &mut self.state {
            RequestState::Uninitialized => self.initialize_request().http_version(),
            RequestState::Request(req) => {
                let req = req.take().unwrap();

                let version = match req.get_version() {
                    fastly::http::Version::HTTP_09 => Some("0.9"),
                    fastly::http::Version::HTTP_10 => Some("1.0"),
                    fastly::http::Version::HTTP_11 => Some("1.1"),
                    fastly::http::Version::HTTP_2 => Some("2.0"),
                    fastly::http::Version::HTTP_3 => Some("3.0"),
                    _ => None,
                };

                self.state = RequestState::Request(Some(req));

                version.map(|v| v.to_string())
            }
            _ => unreachable!(),
        }
    }

    pub fn request_uri(&mut self) -> Option<String> {
        match &mut self.state {
            RequestState::Uninitialized => self.initialize_request().request_uri(),
            RequestState::Request(req) => {
                let req = req.take().unwrap();

                let uri = req.get_url()[Position::BeforePath..].to_string();

                self.state = RequestState::Request(Some(req));

                Some(uri)
            }
            _ => unreachable!(),
        }
    }

    pub fn request_method(&mut self) -> String {
        match &mut self.state {
            RequestState::Uninitialized => self.initialize_request().request_method(),
            RequestState::Request(req) => {
                let req = req.take().unwrap();

                let method = req.get_method_str().to_string();

                self.state = RequestState::Request(Some(req));

                method
            }
            _ => unreachable!(),
        }
    }

    pub fn query_params(&mut self) -> Option<Vec<(String, String)>> {
        match &mut self.state {
            RequestState::Uninitialized => self.initialize_request().query_params(),
            RequestState::Request(req) => {
                let req = req.take().unwrap();

                let query_params = req.get_query::<Vec<(String, String)>>();

                self.state = RequestState::Request(Some(req));

                query_params.ok()
            }
            _ => unreachable!(),
        }
    }

    pub fn read_body_chunk(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        match &mut self.state {
            RequestState::Uninitialized => self.initialize_request().read_body_chunk(buf),
            RequestState::Request(req) => {
                let mut req = req.take().unwrap();

                let body = req.take_body().into_bytes();

                self.state = RequestState::ReadingBody(Some(Cursor::new(body)));

                self.read_body_chunk(buf)
            }
            RequestState::ReadingBody(body) => {
                let mut body = body.take().unwrap();

                body.read(buf).map_err(anyhow::Error::from)
            }
        }
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
