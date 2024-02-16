use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ExpenseSource {
    pub id: i64,
    pub name: String,
    pub expense: RecurringMoneyValue,
}

#[derive(Serialize, Deserialize)]
pub struct RecurringMoneyValue {
    pub amount: i32,
    pub period: Period,
}

#[derive(Serialize, Deserialize)]
pub enum Period {
    Month,
    Year,
}
