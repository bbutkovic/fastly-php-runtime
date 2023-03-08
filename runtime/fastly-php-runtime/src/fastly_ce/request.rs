use fastly::handle::RequestHandle;

pub struct FastlyRequestHandle {
    state: FastlyRequestState,
}

enum FastlyRequestState {
    Uninitialized,
    Request(Option<RequestHandle>),
}

impl FastlyRequestHandle {
    pub fn new() -> Self {
        Self {
            state: FastlyRequestState::Uninitialized,
        }
    }

    fn initialize_request<'a>(&'a mut self) -> &'a mut Self {
        self.state = FastlyRequestState::Request(Some(RequestHandle::from_client()));
        self
    }

    fn initialized_request<'a>(&'a mut self) -> &'a mut Self {
        match &self.state {
            FastlyRequestState::Uninitialized => self.initialize_request(),
            _ => self,
        }
    }

    // pub fn
}
