use std::io::net::ip::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use hyper::server::{Server};

use listener::{Concurrent, ConcurrentHandler};
use middleware::Application;

pub struct Rustless;

impl Rustless {

    pub fn listen(self, app: Application, ip: IpAddr, port: u16) {
        let server = Server::http(ip, port);
        server.listen(Concurrent { handler: Arc::new(app) }).unwrap();
    }

}
