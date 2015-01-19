
use regex::Regex;
use hyper::mime::{Mime, TopLevel, SubLevel};

static MEDIA_REGEX: Regex = regex!(r"vnd\.(?P<vendor>[a-zA-Z_-]+)(?:\.(?P<version>[a-zA-Z0-9]+)(?:\.(?P<param>[a-zA-Z0-9]+))?)?(?:\+(?P<format>[a-zA-Z0-9]+))?");

#[derive(Show)]
pub enum Format {
    JsonFormat,
    PlainTextFormat,
    OtherFormat(Mime)
}

impl Format {
    pub fn from_mime(mime: &Mime) -> Format {
        match mime {
            &Mime(TopLevel::Text, SubLevel::Plain, _) => Format::PlainTextFormat,
            &Mime(TopLevel::Application, SubLevel::Json, _) => Format::JsonFormat,
            _ => Format::OtherFormat(mime.clone())
        }
    }
}

pub struct Media {
    pub vendor: String,
    pub version: Option<String>,
    pub param: Option<String>,
    pub format: Format
}

impl Media {

    pub fn default() -> Media {
        Media::from_mime(&Mime(TopLevel::Text, SubLevel::Plain, vec![]))
    }

    pub fn from_mime(mime: &Mime) -> Media {
        Media {
            vendor: "default".to_string(),
            version: None,
            param: None,
            format: Format::from_mime(mime)
        }
    }

    pub fn from_vendor(mime: &Mime) -> Option<Media> {
        match mime {
            &Mime(TopLevel::Application, SubLevel::Ext(ref ext), _) => {
                match MEDIA_REGEX.captures(ext.as_slice()) {
                    Some(captures) => {
                        let vendor = captures.name("vendor");
                        let version = captures.name("version").map(|s| s.to_string());
                        let param = captures.name("param").map(|s| s.to_string());
                        let format_str = captures.name("format").map(|s| s.to_string());

                        let format = match format_str {
                            Some(format) => if format.as_slice() == "json" { Format::JsonFormat }
                                            else if format.as_slice() == "txt" { Format::PlainTextFormat }
                                            else { Format::from_mime(mime) },
                            None => Format::from_mime(mime)
                        };

                        Some(Media {
                            vendor: vendor.unwrap().to_string(),
                            version: version,
                            param: param,
                            format: format
                        })
                    },
                    None => None
                }
            }
            _ => None
        }
    }
}

#[test]
fn asset_regexp() {
    let captures = MEDIA_REGEX.captures("application/vnd.github.v3.raw+json").unwrap();
    assert_eq!(captures.name("vendor").unwrap(), "github");
    assert_eq!(captures.name("version").unwrap(), "v3");
    assert_eq!(captures.name("param").unwrap(), "raw");
    assert_eq!(captures.name("format").unwrap(), "json");
    
    let captures = MEDIA_REGEX.captures("application/vnd.github.v3+json").unwrap();
    assert_eq!(captures.name("vendor").unwrap(), "github");
    assert_eq!(captures.name("version").unwrap(), "v3");
    assert_eq!(captures.name("param"), None);
    assert_eq!(captures.name("format").unwrap(), "json");
    
    let captures = MEDIA_REGEX.captures("application/vnd.github+json").unwrap();
    assert_eq!(captures.name("vendor").unwrap(), "github");
    assert_eq!(captures.name("version"), None);
    assert_eq!(captures.name("param"), None);
    assert_eq!(captures.name("format").unwrap(), "json");
    
    let captures = MEDIA_REGEX.captures("application/vnd.github").unwrap();
    assert_eq!(captures.name("vendor").unwrap(), "github");
    assert_eq!(captures.name("version"), None);
    assert_eq!(captures.name("param"), None);
    assert_eq!(captures.name("format"), None);
    
    let captures = MEDIA_REGEX.captures("application/vnd");
    assert!(captures.is_none());
}

#[test]
fn assert_media() {

    match Media::from_mime(&"application/json".parse().unwrap()).format {
        Format::JsonFormat => (),
        _ => panic!("Wrong format")
    }

    match Media::from_mime(&"text/plain".parse().unwrap()).format {
        Format::PlainTextFormat => (),
        _ => panic!("Wrong format")
    }

    match Media::from_mime(&"application/octet-stream".parse().unwrap()).format {
        Format::OtherFormat(_) => (),
        _ => panic!("Wrong format")
    }

}
