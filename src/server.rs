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

use std::sync::mpsc::Sender;


const LISTEN_ADDRESS: &'static str = "0.0.0.0:26262";

pub struct Client {
    addr: SocketAddr,
    data: ClientData
}

pub type Clients = Vec<Client>;
pub type ClientData = (f64, f64);

pub fn run(sender: Sender<Vec<ClientData>>) {
    // Create the event loop that will drive this server
    let mut l = Core::new().unwrap();
    let handle = l.handle();
    let mut clients : Clients = vec![];

    let server = SpiroServer::new(&handle);
    l.run(server.for_each(|(addr, words)| {
        if words.len() >= 2 {
            words[0].parse::<f64>().map(|d|
                words[1].parse::<f64>().map(|e|
                    add_to_client_data(addr, (d, e), &mut clients)));
        }

        sender.send(clients.iter().map(|c| c.data).collect());

        Ok(())
    })).unwrap();
}

fn add_to_client_data(addr: SocketAddr, data: ClientData, clients: &mut Clients) {
    for c in clients.iter_mut() {
        if c.addr == addr {
            c.data = data;
            return;
        }
    }
    clients.push(Client { addr: addr, data: data });
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
    type Item = (SocketAddr, Vec<String>);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.inner.poll_read().is_not_ready() {
            return Ok(Async::NotReady)
        }

        loop {
            let mut buf = vec![0; 2048];
            let (size, addr) = try_nb!(self.inner.recv_from(&mut buf));
            buf.truncate(size);

            match String::from_utf8(buf) {
                Ok(s) => return Ok(Some((addr, s.trim().split('\t').map(|s| s.to_owned()).collect())).into()),
                Err(_) => {}
            }
        }
    }
}
