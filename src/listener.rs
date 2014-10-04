use std::io::net::ip::{SocketAddr, IpAddr};
use std::sync::Arc;

use middleware::Application;
use request::Request;

use hyper::server::{Server, Handler, Incoming};
use hyper::server::Request as HyperRequest;
use hyper::server::Response as HyperResponse;
use hyper::header::common::ContentLength;
use hyper::net::{HttpStream, HttpAcceptor, Fresh};

pub trait ConcurrentHandler: Send + Sync {
    fn handle(&self, req: HyperRequest, res: HyperResponse<Fresh>);
}

pub struct Concurrent<H: ConcurrentHandler> { pub handler: Arc<H> }

impl<H: ConcurrentHandler> Handler<HttpAcceptor, HttpStream> for Concurrent<H> {
    fn handle(self, mut incoming: Incoming) {
        for (mut req, mut res) in incoming {
            let clone = self.handler.clone();
            spawn(proc() { clone.handle(req, res) })
        }
    }
}

macro_rules! try_abort(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(..) => return
        }
    }}
)

impl ConcurrentHandler for Application {
    fn handle(&self, mut req: HyperRequest, mut res: HyperResponse<Fresh>) {

        let mut request = Request::wrap(req).unwrap();
        let maybe_response = self.call(&mut request);
        
        match maybe_response {
            Ok(response) => {
                *res.status_mut() = response.status;
                *res.headers_mut() = response.headers;

                match response.body {
                    Some(mut reader) => {
                        let content = try_abort!(reader.read_to_end());
                        res.headers_mut().set(ContentLength(content.len()));
                        let mut res = try_abort!(res.start());
                        try_abort!(res.write(content.as_slice()));
                        try_abort!(res.end());
                    },
                    _ => ()
                }
            },

            Err(_) => println!("No response")
        }
    }
}