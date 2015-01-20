
use serialize::json::{self, ToJson};
use regex;

static MATCHER: regex::Regex = regex!(r":([a-z][a-z_]*)");

pub struct Path {
    regex: regex::Regex,
    pub path: String,
    pub params: Vec<String>
}

impl Path {

    pub fn apply_captures(&self, jobj: &mut json::Object, captures: regex::Captures) {
        for param in self.params.iter() {
            jobj.insert(param.clone(), captures.name(param.as_slice()).unwrap_or("").to_string().to_json());
        }
    }

    pub fn is_match<'a>(&'a self, path: &'a str) -> Option<regex::Captures> {
        match self.regex.captures(path) {
            Some(captures) => Some(captures),
            None => None
        }
    }

    pub fn parse(path: &str, endpoint: bool) -> Result<Path,String> {
        let mut regex_body = "^/".to_string() + Path::sub_regex(path).as_slice();
        
        if endpoint {
            regex_body = regex_body + "$";
        }

        let regex = match regex::Regex::new(regex_body.as_slice()) {
            Ok(re) => re,
            Err(err) => return Err(format!("{}", err))
        };

        let mut params = vec![];
        for capture in MATCHER.captures_iter(path) {
            params.push(capture.at(1).unwrap_or("").to_string()); 
        }

        Ok(Path {
            path: path.to_string(),
            params: params,
            regex: regex
        })
    }

    fn sub_regex(path: &str) -> String {
        return MATCHER.replace_all(path, "(?P<$1>[^/?&]+)");
    }

}

#[test]
fn sub_regex() {
    let res = Path::sub_regex(":user_id/messages/:message_id");
    assert_eq!(res.as_slice(), "(?P<user_id>[^/?&]+)/messages/(?P<message_id>[^/?&]+)")
}

#[test]
fn parse_and_match() {
    let path = Path::parse(":user_id/messages/:message_id", true).unwrap();
    assert!(match path.is_match("/1920/messages/100500") {
        Some(captures) => {
            captures.name("user_id").unwrap() == "1920" && 
            captures.name("message_id").unwrap() == "100500"
        }
        None => false   
    });
    assert!(match path.is_match("/1920/messages/not_match/100500") {
        Some(_) => false,
        None => true
    });
}

#[test]
fn parse_and_match_root() {
    let path = Path::parse("", true).unwrap();
    assert!(match path.is_match("/") {
        Some(captures) => captures.at(0).unwrap() == "/",
        None => false
    });
}

#[test]
fn parse_and_match_single_val() {
    let path = Path::parse(":id", true).unwrap();
    assert!(match path.is_match("/550e8400-e29b-41d4-a716-446655440000") {
        Some(captures) => captures.name("id").unwrap() == "550e8400-e29b-41d4-a716-446655440000",
        None => false
    });
}