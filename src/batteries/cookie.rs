use cookie::{CookieJar};
use middleware::{HandleResult, AfterMiddleware, BeforeMiddleware};
use server::{Request, Response};
use server_backend::header::common::cookie::Cookies;
use server_backend::header::common::set_cookie::SetCookie;

pub trait CookieExt for Sized? {
    fn find_cookie_jar(&mut self) -> Option<&mut CookieJar<'static>>;
    fn store_cookie_jar(&mut self, jar: CookieJar<'static>);
    fn cookies<'a>(&'a mut self) -> &'a mut CookieJar<'static> { 
        self.find_cookie_jar().unwrap()   
    }
}

impl CookieExt for Request {
    fn find_cookie_jar<'a>(&'a mut self) -> Option<&'a mut CookieJar<'static>> {
        self.ext_mut().get_mut()
    }

    fn store_cookie_jar(&mut self, jar: CookieJar<'static>) {
        self.ext_mut().insert(jar);
    }
}

pub struct CookieDecodeMiddleware {
    secret_token: Vec<u8>
}

impl BeforeMiddleware for CookieDecodeMiddleware {
    fn before(&self, req: &mut Request) -> HandleResult<Option<Response>> {
        let token = self.secret_token.as_slice();
        let jar = req.headers().get::<Cookies>()
            .map(|cookies| cookies.to_cookie_jar(token))
            .unwrap_or_else(|| CookieJar::new(token));

        req.ext_mut().insert(jar);
        Ok(None)
    }
}

pub struct CookieEncodeMiddleware;

impl AfterMiddleware for CookieEncodeMiddleware {
    fn after(&self, req: &mut Request, res: &mut Response) -> HandleResult<Option<Response>> {
        let maybe_jar = req.find_cookie_jar();
        match maybe_jar {
            Some(jar) => {
                res.set_header(SetCookie::from_cookie_jar(jar));
            },
            None => ()
        }

        Ok(None)
    }
}

pub fn new(secret_token: &[u8]) -> (CookieDecodeMiddleware, CookieEncodeMiddleware) {
    (
        CookieDecodeMiddleware{secret_token: secret_token.to_vec()},
        CookieEncodeMiddleware
    )
}