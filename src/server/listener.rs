use std::sync::Arc;

use middleware::Application;
use server::request::{Request, ServerRequest};

use server_backend::server::{Connection, Handler, Incoming};
use server_backend::header::common::ContentLength;
use server_backend::net::{HttpStream, HttpAcceptor};

pub trait ConcurrentHandler: Send + Sync {
    fn handle(&self, Connection);
}

pub struct Concurrent<H: ConcurrentHandler> { pub handler: Arc<H> }

impl<H: ConcurrentHandler> Handler<HttpAcceptor, HttpStream> for Concurrent<H> {
    fn handle(self, mut incoming: Incoming) {
        for mut connection in incoming {
            let clone = self.handler.clone();
            spawn(proc() { clone.handle(connection) })
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
    fn handle(&self, connection: Connection) {

        let (req, mut res) = connection.open().unwrap();

        let mut request = ServerRequest::wrap(req).unwrap();
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
                    _ => try_abort!(res.start().and_then(|res| res.end()))
                }
            },

            Err(_) => {
                println!("No response");
                try_abort!(res.start().and_then(|res| res.end()));
            }
        }
    }
}