
use std::io::IoError;
use std::io::fs::PathExtensions;
use std::os::make_absolute;

use middleware::{Handler, HandleResult, NotMatchError, Error};
use server::{Request, Response};

#[deriving(Clone)]
pub struct Static {
    root_path: Path
}

#[deriving(Show)]
pub struct FileError(IoError);

impl Error for FileError {
    fn name(&self) -> &'static str {
        let &FileError(ref error) = self;
        error.desc
    }
}

impl Static {
    pub fn new(root_path: Path) -> Static {
        Static {
            root_path: make_absolute(&root_path)
        }
    }
}

impl Handler for Static {
    fn call(&self, rest_path: &str, _: &mut Request) -> HandleResult<Response> {
        let requested_path = self.root_path.join(Path::new(rest_path.trim_left_chars('/')));

        if requested_path.is_file() {
            match Response::from_file(&requested_path) {
                Ok(response) => {
                    return Ok(response);
                },
                Err(err) => {
                    return Err(FileError(err).abstract());
                }
            }
        }

        Err(NotMatchError.abstract())
    }
}