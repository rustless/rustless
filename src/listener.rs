use std::io::net::ip::{SocketAddr, IpAddr};
use http::server::Server as HttpServer;
use http::server::Config as HttpConfig;
use http::server::request::{Request};
use http::server::response::{ResponseWriter};

#[deriving(Clone)]
pub struct Listener {
	pub ip: IpAddr,
	pub port: u16
}

impl HttpServer for Listener {

	fn get_config(&self) -> HttpConfig {
		HttpConfig {
			bind_address: SocketAddr {
				ip: self.ip,
				port: self.port
			}
		}
	}

	fn handle_request(&self, http_req: Request, http_res: &mut ResponseWriter) {
		println!("Body: {}", http_req.body);
	}
}

impl Listener {

	pub fn serve (&self) {
        self.serve_forever();
    }

}