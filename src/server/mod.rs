use std::io::net::ip::{IpAddr};
use std::sync::Arc;

use server_backend::server::Server as HttpServer;

use server::listener::{Concurrent};
use middleware::Application;

pub use self::request::{Request, ServerRequest};
pub use self::response::Response;

mod listener;
pub mod request;
mod response;

pub struct Server;

impl Server {

    pub fn listen(self, app: Application, ip: IpAddr, port: u16) {
        let server = HttpServer::http(ip, port);
        server.listen(Concurrent { handler: Arc::new(app) }).unwrap();
    }

}