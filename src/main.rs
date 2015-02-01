extern crate hyper;

use std::old_io::net::ip::Ipv4Addr;
use hyper::server::{Server, Request, Response};

// TODO
// - oauth authentication via temporary webserver
fn main() {
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 1337);
    server.listen(hello).unwrap();
}

// Authentication
fn hello(_: Request, mut res: Response) {
    *res.status_mut() = hyper::Ok;
    let mut res = res.start().unwrap();
    res.write(b"Hello World");
    res.end().unwrap();
}
