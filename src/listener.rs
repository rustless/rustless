use std::io::net::ip::{SocketAddr, IpAddr};
use http::server::Server as HttpServer;
use http::server::Config as HttpConfig;
use http::server::request::Request as HttpRequest;
use http::server::response::ResponseWriter as HttpResponseWriter;
use std::sync::Arc;

use middleware::Application;
use request::Request;

#[deriving(Send,Clone)]
pub struct Listener {
	pub ip: IpAddr,
	pub port: u16,
	pub app: Arc<Application>
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

	fn handle_request(&self, http_req: HttpRequest, http_res: &mut HttpResponseWriter) {
		
		let mut request = Request::wrap(http_req).unwrap();
		let response = self.app.call(&mut request);

		match response {
			Ok(response) => {
				response.write(http_res);
			},

			Err(_) => println!("No response")
		}
	}
}

impl Listener {

	pub fn serve (self) {
        self.serve_forever();
    }

}