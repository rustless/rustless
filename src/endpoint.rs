
use serialize::json;
use serialize::Decodable;

struct Endpoint<T> {
	pub desc: &'static str,
	handler: |&params: T|:'static -> String
}

impl<T: Decodable<json::Decoder, json::DecoderError>> Endpoint<T> {
	pub fn decode(from: &str) -> T {
		json::decode(from).unwrap()
	}

	pub fn process(self, params_body: &str) -> String {
		let params = Endpoint::decode(params_body);
		let handler = self.handler;
		handler(params)
	}

	pub fn new(desc: &'static str, handler: |&params: T|:'static -> String) -> Endpoint<T> {
		Endpoint {
			desc: desc,
			handler: handler
		}
	}
}

#[test]
fn params_decode() {
	
	#[deriving(Decodable)]
	struct Params {
		user_id: String,
		user_type: Option<String>
	};

	let endpoint: Endpoint<Params> = Endpoint::new(
		"Test endpoint", 
		|params: Params| -> String {
			assert_eq!(params.user_id.as_slice(), "test");
			assert!(
				match params.user_type {
					Some(String) => false,
					Nothing => true
				}
			)

			"Result".to_string()
		}
	);

	assert_eq!(endpoint.process("{\"user_id\": \"test\"}").as_slice(), "Result");

}