use crate::domain::{ExpenseSource, Period, PeriodKind, RecurringMoneyValue};
use actix_web::web;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    params,
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub fn create_pool() -> r2d2::Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file("cost-of-life.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    pool.get()
        .unwrap()
        .execute(
            "
CREATE TABLE IF NOT EXISTS expense_source (
    id INTEGER PRIMARY KEY,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    expense_amount INTEGER NOT NULL,
    expense_period_kind TEXT CHECK( expense_period_kind IN ('Month', 'Year') ) NOT NULL,
    expense_period_every INTEGER NOT NULL
)
            ",
            (),
        )
        .unwrap();

    pool
}

pub async fn get_expense_source_by_id(
    pool: &Pool,
    owner: &str,
    id: i64,
) -> actix_web::Result<Option<ExpenseSource>> {
    execute(pool, |conn| {
        let mut stmt = conn
            .prepare(
                "SELECT name, expense_amount, expense_period_kind, expense_period_every FROM expense_source WHERE id = ?1 AND owner = ?2",
            )
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;

        match stmt.query_row(params![id, owner], |row| {
            Ok(ExpenseSource {
                id,
                name: row.get(0)?,
                expense: RecurringMoneyValue {
                    amount: row.get(1)?,
                    period: Period {
                        kind: row.get(2)?,
                        every: row.get(3)?,
                    },
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

pub async fn get_all_expense_sources(
    pool: &Pool,
    owner: &str,
) -> actix_web::Result<Vec<ExpenseSource>> {
    execute(pool, |conn| {
        let mut stmt = conn
            .prepare(
                "
SELECT id, name, expense_amount, expense_period_kind, expense_period_every
FROM expense_source
WHERE owner = ?1",
            )
            .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
        stmt.query_map(params![owner], |row| {
            Ok(ExpenseSource {
                id: row.get(0)?,
                name: row.get(1)?,
                expense: RecurringMoneyValue {
                    amount: row.get(2)?,
                    period: Period {
                        kind: row.get(3)?,
                        every: row.get(4)?,
                    },
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
    owner: &str,
    name: &str,
    expense: RecurringMoneyValue,
) -> actix_web::Result<i64> {
    execute(pool, |conn| {
        conn.execute(
            "INSERT INTO expense_source (name, owner, expense_amount, expense_period_kind, expense_period_every) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, owner, expense.amount, expense.period.kind, expense.period.every],
        )
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn delete_expense_source_by_id(
    pool: &Pool,
    owner: &str,
    id: i64,
) -> actix_web::Result<()> {
    execute(pool, |conn| {
        conn.execute(
            "DELETE FROM expense_source WHERE id = ?1 AND owner = ?2",
            params![id, owner],
        )
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
        Ok(())
    })
    .await
}

pub async fn edit_expense_source_by_id(
    pool: &Pool,
    owner: &str,
    id: i64,
    name: &str,
    expense: RecurringMoneyValue,
) -> actix_web::Result<()> {
    execute(pool, |conn| {
        conn.execute(
            "
UPDATE expense_source
SET name=?3,
    expense_amount=?4,
    expense_period_kind=?5,
    expense_period_every=?6
WHERE id=?1 AND owner=?2",
            params![
                id,
                owner,
                name,
                expense.amount,
                expense.period.kind,
                expense.period.every
            ],
        )
        .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
        Ok(())
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

impl ToSql for PeriodKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(match self {
            PeriodKind::Month => b"Month",
            PeriodKind::Year => b"Year",
        })))
    }
}

impl FromSql for PeriodKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        if let ValueRef::Text(period_text) = value {
            return if period_text == b"Month" {
                Ok(PeriodKind::Month)
            } else if period_text == b"Year" {
                Ok(PeriodKind::Year)
            } else {
                Err(rusqlite::types::FromSqlError::InvalidType)
            };
        }

        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}
