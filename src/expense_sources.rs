use crate::{
    domain::{ExpenseSource, RecurringMoneyValue},
    sqlite::Pool,
};
use actix_web::{error, http::header::LOCATION, web, HttpResponse, Responder};
use rusqlite::params;
use serde::Deserialize;

pub fn create_service() -> actix_web::Scope {
    web::scope("/expense/sources")
        .route("", web::post().to(post_expense_sources))
        .route("", web::get().to(get_expense_sources))
        .route("/{id}", web::get().to(get_expense_source_by_id))
}

#[derive(Deserialize)]
struct CreateExpenseSourceRequest {
    name: String,
    expense: RecurringMoneyValue,
}

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
    let result = stmt.query_row([id], |row| {
        Ok(ExpenseSource {
            id,
            name: row.get(0)?,
            expense: RecurringMoneyValue {
                amount: row.get(1)?,
                period: row.get(2)?,
            },
        })
    });

    match result {
        Ok(source) => Ok(HttpResponse::Ok().json(source)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(HttpResponse::NotFound().body(())),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

// #[get("/expense/sources")]
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
