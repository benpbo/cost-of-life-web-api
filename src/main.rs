use actix_web::{web, App, HttpServer};
use r2d2_sqlite::{self, SqliteConnectionManager};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::expense_sources::create_service as create_expense_sources_service;

mod domain;
mod expense_sources;
mod sqlite;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // connect to SQLite DB
    let manager = SqliteConnectionManager::file("cost-of-life.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    pool.get()
        .unwrap()
        .execute(
            "
CREATE TABLE IF NOT EXISTS expense_source (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    expense_amount INTEGER NOT NULL,
    expense_period TEXT CHECK( expense_period IN ('Month', 'Year') ) NOT NULL
)
            ",
            (),
        )
        .unwrap();

    let bind_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080);
    log::info!("Starting HTTP server at http://{bind_address}");
    HttpServer::new(move || {
        App::new()
            .service(create_expense_sources_service())
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(bind_address)?
    .run()
    .await
}
