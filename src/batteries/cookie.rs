use cookie::{CookieJar};
use backend::{Request};
use {Extensible};
use server::header::common::cookie::Cookies;
use server::header::common::set_cookie::SetCookie;

use iron::{AfterMiddleware, BeforeMiddleware, IronResult};

use iron::Request as IronRequest;
use iron::Response as IronResponse;

struct Jar;

impl ::typemap::Assoc<CookieJar<'static>> for Jar {}

pub trait CookieExt for Sized? {
    fn find_cookie_jar(&mut self) -> Option<&mut CookieJar<'static>>;
    fn store_cookie_jar(&mut self, jar: CookieJar<'static>);
    fn cookies<'a>(&'a mut self) -> &'a mut CookieJar<'static> { 
        self.find_cookie_jar().unwrap()   
    }
}

impl CookieExt for Request {
    fn find_cookie_jar<'a>(&'a mut self) -> Option<&'a mut CookieJar<'static>> {
        self.ext_mut().get_mut::<Jar, CookieJar<'static>>()
    }

    fn store_cookie_jar(&mut self, jar: CookieJar<'static>) {
        self.ext_mut().insert::<Jar, CookieJar<'static>>(jar);
    }
}

pub struct CookieDecodeMiddleware {
    secret_token: Vec<u8>
}

impl BeforeMiddleware for CookieDecodeMiddleware {
    fn before(&self, req: &mut IronRequest) -> IronResult<()> {
        let token = self.secret_token.as_slice();
        let jar = req.headers().get::<Cookies>()
            .map(|cookies| cookies.to_cookie_jar(token))
            .unwrap_or_else(|| CookieJar::new(token));

        req.ext_mut().insert(jar);
        Ok(())
    }
}

pub struct CookieEncodeMiddleware;

impl AfterMiddleware for CookieEncodeMiddleware {
    fn after(&self, req: &mut IronRequest, res: &mut IronResponse) -> IronResult<()> {
        let maybe_jar = (req as &mut Request).find_cookie_jar();
        match maybe_jar {
            Some(jar) => {
                res.headers.set(SetCookie::from_cookie_jar(jar));
            },
            None => ()
        }

        Ok(())
    }
}

pub fn new(secret_token: &[u8]) -> (CookieDecodeMiddleware, CookieEncodeMiddleware) {
    (
        CookieDecodeMiddleware{secret_token: secret_token.to_vec()},
        CookieEncodeMiddleware
    )
}