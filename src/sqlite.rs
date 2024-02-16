use crate::domain::{ExpenseSource, Period, RecurringMoneyValue};
use actix_web::web;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    params,
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub async fn get_expense_source_by_id(
    pool: &Pool,
    id: i64,
) -> actix_web::Result<Option<ExpenseSource>> {
    execute(pool, |conn| {
        let mut stmt = conn
            .prepare(
                "SELECT name, expense_amount, expense_period FROM expense_source WHERE id = ?1",
            )
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

        match stmt.query_row([id], |row| {
            Ok(ExpenseSource {
                id,
                name: row.get(0)?,
                expense: RecurringMoneyValue {
                    amount: row.get(1)?,
                    period: row.get(2)?,
                },
            })
        }) {
            Ok(source) => Ok(Some(source)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
        }
    })
    .await
}

pub async fn get_all_expense_sources(pool: &Pool) -> actix_web::Result<Vec<ExpenseSource>> {
    execute(pool, |conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, expense_amount, expense_period FROM expense_source")
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
        stmt.query_map([], |row| {
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
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))
    })
    .await
}

pub async fn create_expense_source(
    pool: &Pool,
    name: &str,
    expense: RecurringMoneyValue,
) -> actix_web::Result<i64> {
    execute(pool, |conn| {
        conn.execute(
            "INSERT INTO expense_source (name, expense_amount, expense_period) VALUES (?1, ?2, ?3)",
            params![name, expense.amount, expense.period],
        )
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

async fn execute<
    T,
    F: FnOnce(PooledConnection<SqliteConnectionManager>) -> actix_web::Result<T>,
>(
    pool: &Pool,
    f: F,
) -> actix_web::Result<T> {
    let pool = pool.clone();
    let conn = web::block(move || pool.get())
        .await?
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

    f(conn)
}

impl ToSql for Period {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            Period::Month => b"Month",
            Period::Year => b"Year",
        })))
    }
}

impl FromSql for Period {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        if let ValueRef::Text(period_text) = value {
            return if period_text == b"Month" {
                Ok(Period::Month)
            } else if period_text == b"Year" {
                Ok(Period::Year)
            } else {
                Err(rusqlite::types::FromSqlError::InvalidType)
            };
        }

        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}
