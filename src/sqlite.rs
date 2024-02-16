use crate::domain::Period;
use rusqlite::{
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

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
