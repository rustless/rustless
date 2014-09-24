
use serialize::json;
use regex::{Regex, Captures};

#[deriving(Send)]
pub struct Route {
	pub matcher: ||:'static + Sync + Send -> Result<json::Json, String>
}

static matcher: Regex = regex!(r":([a-z][a-z_]*)");

struct Path {
	regex: Regex
}

impl Path {

	pub fn is_match<'a>(&'a self, path: &'a str) -> Option<Captures> {
		self.regex.captures(path)
	}

	pub fn parse(path: &str) -> Result<Path,String> {
		let regex = match Regex::new(Path::sub_regex(path).as_slice()) {
			Ok(re) => re,
			Err(err) => return Err(format!("{}", err))
		};

		Ok(Path {
			regex: regex
		})
	}

	fn sub_regex(path: &str) -> String {
		return format!("^{}$", matcher.replace_all(path, "(?P<$1>[^/?&]+)"));
	}

}

#[test]
fn sub_regex() {
	let res = Path::sub_regex(":user_id/messages/:message_id");
	assert_eq!(res.as_slice(), "^(?P<user_id>[^/?&]+)/messages/(?P<message_id>[^/?&]+)$")
}

#[test]
fn parse_and_match() {
	let path = Path::parse(":user_id/messages/:message_id").unwrap();
	assert!(match path.is_match("1920/messages/100500") {
		Some(captures) => {
			captures.name("user_id") == "1920" && 
			captures.name("message_id") == "100500"
		}
		None => false	
	});
	assert!(match path.is_match("1920/messages/not_match/100500") {
		Some(_) => false,
		None => true
	});
}

#[root]
fn parse_and_match_root() {
	let path = Path::parse("").unwrap();
	assert!(match path.is_match("") {
		Some(captures) => captures.at(0) == "",
		None => false
	});
}

#[root]
fn parse_and_match_single_val() {
	let path = Path::parse(":id").unwrap();
	assert!(match path.is_match("550e8400-e29b-41d4-a716-446655440000") {
		Some(captures) => captures.name("id") == "550e8400-e29b-41d4-a716-446655440000",
		None => false
	});
}