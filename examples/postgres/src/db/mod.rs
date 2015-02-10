use rustless::{self, Extensible};

// A set of helpful types we need to shorten things
pub use deuterium_orm::adapter::postgres::PostgresPool;
pub use deuterium_orm::adapter::postgres::PostgresPooledConnection;
pub use deuterium_orm::adapter::postgres::setup;

pub use postgres::GenericConnection as Connection;

pub mod migrations;

// A unit struct we need to store our pool into TypeMap
// See `typemap` docs for clarification.
pub struct AppDb;
impl ::typemap::Key for AppDb {
    type Value = PostgresPool;
}

// Let's create a shorthand function to simplify a DB access within our
// endpoints.
pub trait DatabaseExt: rustless::Extensible {
    fn db(&self) -> PostgresPooledConnection;
}
impl DatabaseExt for rustless::Application {
    fn db(&self) -> PostgresPooledConnection {
        self.ext().get::<AppDb>().unwrap().get().unwrap()
    }
}