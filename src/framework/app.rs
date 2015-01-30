use typemap;
use framework::api;
use backend;

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

    pub fn call<'a>(&self, req: &'a mut (backend::Request + 'a)) -> backend::HandleExtendedResult<backend::Response> {
        self.root_api.call((req.url().path().connect("/").as_slice()).as_slice(), req, self)
    }
}