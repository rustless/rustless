use cookie::{CookieJar};
use server::Request;

pub trait Cookies {
    fn find_cookie_jar(&self) -> Option<&CookieJar<'static>>;
    fn store_cookie_jar(&mut self, jar: CookieJar<'static>);
    fn cookies<'a>(&'a mut self, key: &[u8]) -> &'a CookieJar {
        let has_jar = { self.find_cookie_jar().is_some() };

        if !has_jar {
            let jar = CookieJar::new(key);
            self.store_cookie_jar(jar);    
        }
           
        self.find_cookie_jar().unwrap()   
    }
}

impl Cookies for &'static mut Request {
    fn find_cookie_jar(&self) -> Option<&CookieJar<'static>> {
        self.ext().find()
    }

    fn store_cookie_jar(&mut self, jar: CookieJar<'static>) {
        self.ext_mut().insert(jar);
    }
}