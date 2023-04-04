// pub trait State {}

// pub struct State {

// }

// pub struct ResponseHandler {
//     state: State,
// }

// impl ResponseHandler {
//     pub fn send_response_code(code: usize, status: Option<String>) -> anyhow::Result<()> {
//         todo!()
//     }

//     pub fn send_header(key: String, value: String) -> anyhow::Result<()> {
//         todo!()
//     }

//     pub fn stream_body(body: String) -> anyhow::Result<()> {
//         todo!()
//     }

//     pub fn finish() -> anyhow::Result<()> {
//         todo!()
//     }
// }

// // trait Transition<F, T> {
// //     pub fn transition()
// // }

// pub struct InitialState {}

// pub struct SendingHeadersState {}

// pub struct StreamingBodyState {}

// enum State {
//     Initial(InitialState),
//     SendingHeaders(SendingHeadersState),
//     StreamingBody(StreamingBodyState),
//     Finished,
// }

// ---------------------------------------------------------------------------------------------------------------------

pub struct Response<S> {
    state: S,
}

pub struct Initial {}

pub struct SendingHeaders {}

pub struct StreamingBody {}

pub struct Finished {}

impl Response<Initial> {
    fn new() -> Self {
        Response { state: Initial {} }
    }
}

impl From<Response<Initial>> for Response<SendingHeaders> {
    fn from(value: Response<Initial>) -> Self {
        todo!()
    }
}

impl From<Response<SendingHeaders>> for Response<StreamingBody> {
    fn from(value: Response<SendingHeaders>) -> Self {
        todo!()
    }
}

mod simulation {
    use super::*;

    fn simulate() {
        let state = Response::new();

        let state: Response<Initial> = state.into();

        let state: Response<StreamingBody> = state.into();
    }
}
