use fastly::handle::RequestHandle as FastlyRequestHandle;

pub struct RequestHandle {
    state: RequestState,
}

enum RequestState {
    Uninitialized,
    Request(Option<FastlyRequestHandle>),
}

impl RequestHandle {
    pub fn new() -> Self {
        Self {
            state: RequestState::Uninitialized,
        }
    }

    fn initialize_request<'a>(&'a mut self) -> &'a mut Self {
        self.state = RequestState::Request(Some(FastlyRequestHandle::from_client()));
        self
    }

    fn initialized_request<'a>(&'a mut self) -> &'a mut Self {
        match &self.state {
            RequestState::Uninitialized => self.initialize_request(),
            _ => self,
        }
    }
}
