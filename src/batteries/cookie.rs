use iron;

use cookie;
use backend::{self, Request};
use server::header;
use ::{Extensible};

struct CookieJarKey;

impl ::typemap::Key for CookieJarKey {
    type Value = cookie::CookieJar<'static>;
}

pub trait CookieExt {
    fn find_cookie_jar(&mut self) -> Option<&mut cookie::CookieJar<'static>>;
    fn store_cookie_jar(&mut self, jar: cookie::CookieJar<'static>);
    fn cookies<'a>(&'a mut self) -> &'a mut cookie::CookieJar<'static> { 
        self.find_cookie_jar().unwrap()   
    }
}

impl CookieExt for backend::Request {
    fn find_cookie_jar<'a>(&'a mut self) -> Option<&'a mut cookie::CookieJar<'static>> {
        self.ext_mut().get_mut::<CookieJarKey>()
    }

    fn store_cookie_jar(&mut self, jar: cookie::CookieJar<'static>) {
        self.ext_mut().insert::<CookieJarKey>(jar);
    }
}

pub struct CookieDecodeMiddleware {
    secret_token: Vec<u8>
}

impl iron::BeforeMiddleware for CookieDecodeMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {
        let token = self.secret_token.as_slice();
        let jar = req.headers().get::<header::Cookies>()
            .map(|cookies| cookies.to_cookie_jar(token))
            .unwrap_or_else(|| cookie::CookieJar::new(token));

        req.ext_mut().insert::<CookieJarKey>(jar);
        Ok(())
    }
}

#[allow(missing_copy_implementations)]
pub struct CookieEncodeMiddleware;

impl iron::AfterMiddleware for CookieEncodeMiddleware {
    fn after(&self, req: &mut iron::Request, res: &mut iron::Response) -> iron::IronResult<()> {
        let maybe_jar = (req as &mut backend::Request).find_cookie_jar();
        match maybe_jar {
            Some(jar) => {
                res.headers.set(header::SetCookie::from_cookie_jar(jar));
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