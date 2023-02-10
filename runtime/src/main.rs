use fastly::{Error, Request, Response as FastlyResponse};
use response::Response;

mod archive;
mod noops;
mod response;
mod runtime;
mod util;

#[fastly::main]
fn main(_req: Request) -> Result<FastlyResponse, Error> {
    hello_world_if_empty();
    println!("req");
    archive::phar::execute_loaded();
    Ok(FastlyResponse::from_body("Hello, world!"))
}

fn hello_world_if_empty() {
    archive::phar::store_from_bytes(bytes::Bytes::from("<?php echo 'Hello, world!'; "));
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    archive::phar::store_from_stdin();
}
