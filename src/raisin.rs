use std::io::net::ip::{IpAddr};
use std::sync::Arc;

use listener::{Listener};
use middleware::Application;

pub struct Raisin {
	pub some: u16,
}

impl Raisin {

	pub fn listen(self, app: Application, ip: IpAddr, port: u16) {
		spawn(proc() {
			let arc_app = Arc::new(app);
			Listener {
				ip: ip,
				port: port,
				app: arc_app
			}.serve();
		})
	}

}
