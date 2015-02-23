#![feature(env)]
#![feature(plugin)]
#![feature(core)]
#![feature(old_io)]
#![feature(old_path)]

#![plugin(deuterium_plugin)]
#![plugin(docopt_macros)]

extern crate postgres;
#[macro_use] #[no_link] extern crate deuterium_plugin;
#[macro_use] extern crate deuterium_orm;
extern crate rustless;
extern crate typemap;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;
extern crate iron;
extern crate docopt;
extern crate uuid;
extern crate jsonway;
extern crate valico;
extern crate url;

use std::env;
use rustless::prelude::*;
use rustless::batteries::schemes;
use rustless::batteries::swagger;
use valico::json_schema;
use std::old_io::net::ip;

use self::db::DatabaseExt;

mod db;
mod serializers;
mod models;
mod api;

docopt!(Args derive Debug, "
Example backend.

Usage:
  backend run
  backend g migration <migration-name>
  backend db migrate [<version>]
  backend db rollback [<steps>]
  backend --version

Options:
  -h --help        Show this screen.
  --version        Show version.
  --ip=<ip>        Specify server ip [default: 127.0.0.1]
  --port=<port>    Specify server port [default: 3001]
");

fn run_db(app: &mut rustless::Application) {
    let connection_str = env::var("POSTGRES_CONNECTION")
        .ok().expect("Provide POSTGRES_CONNECTION environment variable");
    let pool = self::db::setup(&connection_str[..], 5);

    // Here we use TypeMap to store out database pool
    app.ext.insert::<self::db::AppDb>(pool);
}

#[allow(dead_code)]
fn main() {
    let mut app = rustless::Application::new(self::api::root());

    swagger::enable(&mut app, swagger::Spec {
        info: swagger::Info {
            title: "Example tweet API".to_string(),
            description: Some("Simple API to demonstration".to_string()),
            contact: Some(swagger::Contact {
                name: "Stanislav Panferov".to_string(),
                url: Some("http://panferov.me".to_string()),
                ..std::default::Default::default()
            }),
            license: Some(swagger::License {
                name: "MIT".to_string(),
                url: "http://opensource.org/licenses/MIT".to_string()
            }),
            ..std::default::Default::default()
        },
        ..std::default::Default::default()
    });

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    if args.flag_version {

        println!("0.0.0-dev")

    } else if args.cmd_run {

        app.root_api.mount(swagger::create_api("api-docs"));
        schemes::enable_schemes(&mut app, json_schema::Scope::new()).unwrap();

        run_db(&mut app);

        // See `iron` docs for clarification.
        let chain = iron::Chain::new(app);

        let host: ip::IpAddr = args.flag_ip.parse().unwrap();
        let port: u16 = args.flag_port.parse().unwrap();

        iron::Iron::new(chain).http((host, port)).unwrap();

        println!("On {}", port);

    } else if args.cmd_db && args.cmd_migrate {

        run_db(&mut app);
        deuterium_orm::migration::run(&db::migrations::migrations(), &*app.db());

    } else if args.cmd_db && args.cmd_rollback {

        run_db(&mut app);
        deuterium_orm::migration::rollback(
          args.arg_steps.parse().unwrap_or(1),
          &db::migrations::migrations(),
          &*app.db()
        );

    } else if args.cmd_g && args.cmd_migration {

        let name = deuterium_orm::migration::create_migration_file(
          &args.arg_migration_name[..],
          Path::new("src/db/migrations")
        );

        println!("Migration with name {} was generated.", name);

    }
}