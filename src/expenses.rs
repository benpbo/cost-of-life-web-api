use crate::common::RecurringMoneyValue;
use serde::Serialize;

#[derive(Serialize)]
pub struct ExpenseSource {
    pub name: String,
    pub expense: RecurringMoneyValue,
}
