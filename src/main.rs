use crate::common::Period;
use actix_web::{get, web, App, HttpServer, Responder};
use common::RecurringMoneyValue;
use expenses::ExpenseSource;

mod common;
mod expenses;

#[get("/expense/sources")]
async fn get_expense_sources() -> impl Responder {
    let sources = vec![
        ExpenseSource {
            name: "10 Per Year".to_string(),
            expense: RecurringMoneyValue {
                amount: 10,
                period: Period::Year,
            },
        },
        ExpenseSource {
            name: "10 Per Month".to_string(),
            expense: RecurringMoneyValue {
                amount: 10,
                period: Period::Month,
            },
        },
    ];

    web::Json(sources)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_expense_sources))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
