extern crate futures;
#[macro_use] extern crate tokio_core;

mod server;

fn main() {
    server::run();
}
