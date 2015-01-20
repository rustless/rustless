use typemap;
use framework::api;
use backend;
use server::status;
use errors;

pub struct Application {
    pub ext: typemap::TypeMap,
    pub root_api: api::Api 
}

unsafe impl Send for Application {}

impl Application {
    pub fn new(root_api: api::Api) -> Application {
        Application {
            root_api: root_api,
            ext: typemap::TypeMap::new()
        }
    }

    pub fn call(&self, req: &mut backend::Request) -> backend::HandleResult<backend::Response> {
        self.root_api.call(("/".to_string() + req.url().path().connect("/").as_slice()).as_slice(), req, self)
    }

    pub fn call_with_not_found(&self, req: &mut backend::Request) -> backend::HandleResult<backend::Response> {
        let res = self.call(req);
        match res {
            Ok(res) => Ok(res),
            Err(err) => {
                if err.downcast::<errors::NotMatch>().is_some() {
                    return Ok(backend::Response::from_string(status::StatusCode::NotFound, "".to_string()));
                }
                return Err(err);
            }
        }
    }
}