#![feature(env)]

extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rustless;
extern crate typemap;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;
extern crate iron;

use std::env;
use rustc_serialize::json;

use rustless::prelude::*;

// A set of helpful types we need to shorten things
pub type PostgresPool = r2d2::Pool<r2d2_postgres::PostgresConnectionManager>;
pub type PostgresPooledConnection<'a> = r2d2::PooledConnection<'a, r2d2_postgres::PostgresConnectionManager>;

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

/// Our main model
#[derive(Debug, RustcEncodable)]
pub struct Jedi {
    id: i32,
    name: String,
    force_level: i32
}

/// This function creates a pool of connections to PostgreSQL database
/// using `r2d2_postgres` functionality. See `r2d2_postgres` docs for clarification.
#[allow(dead_code)]
pub fn setup_db(connection_str: &str, pool_size: u32) -> PostgresPool {
    let manager = r2d2_postgres::PostgresConnectionManager::new(connection_str, postgres::SslMode::None);
    let config = r2d2::Config {
        pool_size: pool_size,
        test_on_check_out: true,
        ..::std::default::Default::default()
    };

    let handler = Box::new(r2d2::NoopErrorHandler);
    r2d2::Pool::new(config, manager, handler).unwrap()
}

/// The function creates and fills the main table of our data.
fn setup_tables(cn: &postgres::GenericConnection) {
   cn.batch_execute(r#"
        DROP TABLE IF EXISTS jedi CASCADE;
        CREATE TABLE jedi (
            id          serial PRIMARY KEY,
            name        varchar(40) NOT NULL,
            force_level integer NOT NULL
        );

        INSERT INTO jedi (name, force_level) VALUES
            ('Luke Skywalker', 100),
            ('Mace Windu', 90),
            ('Obi-Wan Kenoby', 99),
            ('Kit Fisto', 70),
            ('Count Dooku', 99),
            ('Darth Maul', 70),
            ('Anakin Skywalker', 100);

    "#).unwrap();
}

#[allow(dead_code)]
fn main() {
    let api = rustless::Api::build(|api| {
        api.prefix("api");
        api.version("v1", rustless::Versioning::Path);
        
        api.get("jedi", |endpoint| {
            endpoint.handle(|mut client, _params| {
                // Note that .db() is an extension methods that we created with DatabaseExt
                let cn = client.app.db();

                // See `rust-postgres` docs for clarification.
                let statement = cn.prepare("SELECT id, name, force_level FROM jedi;").ok().expect("Connection is broken");
                let jedi_list: Vec<Jedi> = statement.query(&[]).unwrap().map(|row| {
                    Jedi {
                        id: row.get(0),
                        name: row.get(1),
                        force_level: row.get(2)
                    }
                }).collect();

                // Here we set right content type and send our answer back to the client
                client.set_json_content_type();
                client.text(json::encode(&jedi_list).ok().unwrap())
            })
        });
    });

    let mut app = rustless::Application::new(api);
    let connection_str = env::var_string("POSTGRES_CONNECTION").ok().expect("Provide POSTGRES_CONNECTION environment variable");
    let pool = setup_db(&connection_str[], 5);

    {
        // We create a new scope because we need the connection only for `setup_tables(..)` call
        let connection = pool.get().unwrap();
        setup_tables(&*connection);
    }

    // Here we use TypeMap to store out database pool
    app.ext.insert::<AppDb>(pool);

    // See `iron` docs for clarification.
    let chain = iron::Chain::new(app);
    iron::Iron::new(chain).listen("localhost:4000").unwrap();

    println!("On 4000");
}