
use serialize::json;
use serialize::Decodable;

struct Endpoint<T> {
	pub desc: &'static str,
	params: T 
}

impl<T: Decodable<json::Decoder, json::DecoderError>> Endpoint<T> {
	pub fn decode(from: &str) -> T {
		json::decode(from).unwrap()
	}

	pub fn new(desc: &'static str, params_body: &str) -> Endpoint<T> {
		Endpoint {
			desc: desc,
			params: Endpoint::decode(params_body)
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

	let endpoint: Endpoint<Params> = Endpoint::new("Test endpoint", "{\"user_id\": \"test\"}");

	assert_eq!(endpoint.params.user_id.as_slice(), "test");
	assert!(
		match endpoint.params.user_type {
			Some(String) => false,
			Nothing => true
		}
	)

}