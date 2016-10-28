//! An echo server that just writes back everything that's written to it.

use std::env;
use std::net::SocketAddr;
use std::io;

use futures;
use futures::{Future,Poll,Async};
use futures::stream::Stream;
use tokio_core::io::{copy, Io};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::{Core, Handle};

const LISTEN_ADDRESS: &'static str = "0.0.0.0:26262";

pub fn run() {
    // Create the event loop that will drive this server
    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let server = SpiroServer::new(&handle);
    l.run(server.for_each(|x| {
        println!("{:?}", x);
        Ok(())
    })).unwrap();
}


struct SpiroServer {
    inner: UdpSocket,
}

impl SpiroServer {
    pub fn new(handle: &Handle) -> SpiroServer {
        let addr = LISTEN_ADDRESS.parse::<SocketAddr>().unwrap();
        println!("Listening on: {}", addr);
        SpiroServer { inner: UdpSocket::bind(&addr, handle).unwrap() }
    }
}

impl Stream for SpiroServer {
    type Item = (SocketAddr, Vec<u8>);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.inner.poll_read().is_not_ready() {
            return Ok(Async::NotReady)
        }

        let mut buf = vec![0; 1024];
        let (size, addr) = try_nb!(self.inner.recv_from(&mut buf));
        buf.truncate(size);
        if size == 0 {
            Ok(None.into())
        } else {
            Ok(Some((addr, buf)).into())
        }
    }
}

