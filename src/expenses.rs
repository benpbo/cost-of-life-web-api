use crate::common::RecurringMoneyValue;
use serde::Serialize;

#[derive(Serialize)]
pub struct ExpenseSource {
    pub id: i64,
    pub name: String,
    pub expense: RecurringMoneyValue,
}
