use crate::{
    domain::RecurringMoneyValue,
    jwt::Claims,
    sqlite::{
        create_expense_source, delete_expense_source_by_id, edit_expense_source_by_id,
        get_all_expense_sources, get_expense_source_by_id, Pool,
    },
};
use actix_web::{
    http::header::LOCATION,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use jsonwebtoken::TokenData;
use serde::Deserialize;

pub fn create_service() -> actix_web::Scope {
    web::scope("/expense/sources")
        .route("", web::post().to(post_expense_sources))
        .route("", web::get().to(get_expense_sources))
        .route("/{id}", web::get().to(get_expense_source))
        .route("/{id}", web::delete().to(delete_expense_source))
        .route("/{id}", web::put().to(put_expense_source))
}

#[derive(Deserialize, Clone, Debug)]
struct CreateExpenseSourceRequest {
    name: String,
    expense: RecurringMoneyValue,
}

#[derive(Deserialize, Clone, Debug)]
struct EditExpenseSourceRequest {
    name: String,
    expense: RecurringMoneyValue,
}

async fn post_expense_sources(
    db: web::Data<Pool>,
    token: ReqData<TokenData<Claims>>,
    expense_source: web::Json<CreateExpenseSourceRequest>,
) -> actix_web::Result<impl Responder> {
    let id = create_expense_source(
        &db,
        &token.claims.sub,
        &expense_source.name,
        expense_source.expense,
    )
    .await?;
    Ok(HttpResponse::Created()
        .insert_header((LOCATION, format!("/expense/sources/{id}")))
        .body(()))
}

async fn get_expense_source(
    db: web::Data<Pool>,
    token: ReqData<TokenData<Claims>>,
    id: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    match get_expense_source_by_id(&db, &token.claims.sub, id.into_inner()).await? {
        Some(expense_source) => Ok(HttpResponse::Ok().json(expense_source)),
        None => Ok(HttpResponse::NotFound().body(())),
    }
}

async fn get_expense_sources(
    db: web::Data<Pool>,
    token: ReqData<TokenData<Claims>>,
) -> actix_web::Result<impl Responder> {
    let sources = get_all_expense_sources(&db, &token.claims.sub).await?;
    Ok(web::Json(sources))
}

async fn delete_expense_source(
    db: web::Data<Pool>,
    token: ReqData<TokenData<Claims>>,
    id: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    delete_expense_source_by_id(&db, &token.claims.sub, id.into_inner()).await?;
    Ok(HttpResponse::NoContent())
}

async fn put_expense_source(
    db: web::Data<Pool>,
    token: ReqData<TokenData<Claims>>,
    id: web::Path<i64>,
    expense_source: web::Json<EditExpenseSourceRequest>,
) -> actix_web::Result<impl Responder> {
    edit_expense_source_by_id(
        &db,
        &token.claims.sub,
        id.into_inner(),
        &expense_source.name,
        expense_source.expense,
    )
    .await?;
    Ok(HttpResponse::NoContent())
}
