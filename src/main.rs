use actix_web::{
    error, get, http::header::LOCATION, post, web, App, HttpResponse, HttpServer, Responder,
};
use common::RecurringMoneyValue;
use expenses::ExpenseSource;
use r2d2_sqlite::{self, SqliteConnectionManager};
use rusqlite::params;
use serde::Deserialize;
use sqlite::Pool;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod common;
mod expenses;
mod sqlite;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // connect to SQLite DB
    let manager = SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager).unwrap();

    pool.get()
        .unwrap()
        .execute(
            "
CREATE TABLE expense_source (
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
            .service(post_expense_sources)
            .service(get_expense_sources)
            .service(get_expense_source_by_id)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(bind_address)?
    .run()
    .await
}

#[derive(Deserialize)]
struct CreateExpenseSourceRequest {
    name: String,
    expense: RecurringMoneyValue,
}

#[post("/expense/sources")]
async fn post_expense_sources(
    db: web::Data<Pool>,
    expense_source: web::Json<CreateExpenseSourceRequest>,
) -> actix_web::Result<impl Responder> {
    let conn = web::block(move || db.get())
        .await?
        .map_err(|err| error::ErrorInternalServerError(err))?;

    conn.execute(
        "INSERT INTO expense_source (name, expense_amount, expense_period) VALUES (?1, ?2, ?3)",
        params![
            expense_source.name,
            expense_source.expense.amount,
            expense_source.expense.period
        ],
    )
    .map_err(|err| error::ErrorInternalServerError(err))?;
    let id = conn.last_insert_rowid();

    Ok(HttpResponse::Created()
        .insert_header((LOCATION, format!("/expense/sources/{id}")))
        .body(()))
}

#[get("/expense/sources/{id}")]
async fn get_expense_source_by_id(
    db: web::Data<Pool>,
    id: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    let conn = web::block(move || db.get())
        .await?
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let mut stmt = conn
        .prepare("SELECT name, expense_amount, expense_period FROM expense_source WHERE id = ?1")
        .map_err(|err| error::ErrorInternalServerError(err))?;
    let id = id.into_inner();
    let source = stmt
        .query_row([id], |row| {
            Ok(ExpenseSource {
                id,
                name: row.get(0)?,
                expense: RecurringMoneyValue {
                    amount: row.get(1)?,
                    period: row.get(2)?,
                },
            })
        })
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().json(source))
}

#[get("/expense/sources")]
async fn get_expense_sources(db: web::Data<Pool>) -> actix_web::Result<impl Responder> {
    let conn = web::block(move || db.get())
        .await?
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let mut stmt = conn
        .prepare("SELECT id, name, expense_amount, expense_period FROM expense_source")
        .map_err(|err| error::ErrorInternalServerError(err))?;
    let sources: Vec<_> = stmt
        .query_map([], |row| {
            Ok(ExpenseSource {
                id: row.get(0)?,
                name: row.get(1)?,
                expense: RecurringMoneyValue {
                    amount: row.get(2)?,
                    period: row.get(3)?,
                },
            })
        })
        .and_then(Iterator::collect)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(web::Json(sources))
}
