use actix_web::{web, App, HttpServer};
use actix_web_httpauth::{
    extractors::bearer::Config as BearerConfig, middleware::HttpAuthentication,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::expense_sources::create_service as create_expense_sources_service;

mod domain;
mod expense_sources;
mod jwt;
mod sqlite;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // connect to SQLite DB
    let bind_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080);
    log::info!("Starting HTTP server at http://{bind_address}");
    HttpServer::new(move || {
        App::new()
            .app_data(BearerConfig::default().scope("openid profile email"))
            .wrap(HttpAuthentication::bearer(crate::jwt::validate))
            .service(create_expense_sources_service())
            .app_data(web::Data::new(sqlite::create_pool()))
    })
    .bind(bind_address)?
    .run()
    .await
}
