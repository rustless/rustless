use std::io::net::ip::{IpAddr};
use listener::{Listener};

pub struct Raisin {
	pub some: u16,
}

impl Raisin {

	pub fn listen(self, ip: IpAddr, port: u16) {
		spawn(proc() {
			Listener {
				ip: ip,
				port: port
			}.serve();
		})
	}

}
