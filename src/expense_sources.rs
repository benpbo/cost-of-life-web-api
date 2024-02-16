use crate::{
    domain::RecurringMoneyValue,
    sqlite::{create_expense_source, get_all_expense_sources, get_expense_source_by_id, Pool},
};
use actix_web::{http::header::LOCATION, web, HttpResponse, Responder};
use serde::Deserialize;

pub fn create_service() -> actix_web::Scope {
    web::scope("/expense/sources")
        .route("", web::post().to(post_expense_sources))
        .route("", web::get().to(get_expense_sources))
        .route("/{id}", web::get().to(get_expense_source))
}

#[derive(Deserialize, Clone)]
struct CreateExpenseSourceRequest {
    name: String,
    expense: RecurringMoneyValue,
}

async fn post_expense_sources(
    db: web::Data<Pool>,
    expense_source: web::Json<CreateExpenseSourceRequest>,
) -> actix_web::Result<impl Responder> {
    let id = create_expense_source(&db, &expense_source.name, expense_source.expense).await?;
    Ok(HttpResponse::Created()
        .insert_header((LOCATION, format!("/expense/sources/{id}")))
        .body(()))
}

async fn get_expense_source(
    db: web::Data<Pool>,
    id: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    match get_expense_source_by_id(&db, id.into_inner()).await? {
        Some(expense_source) => Ok(HttpResponse::Ok().json(expense_source)),
        None => Ok(HttpResponse::NotFound().body(())),
    }
}

// #[get("/expense/sources")]
async fn get_expense_sources(db: web::Data<Pool>) -> actix_web::Result<impl Responder> {
    let sources = get_all_expense_sources(&db).await?;
    Ok(web::Json(sources))
}
