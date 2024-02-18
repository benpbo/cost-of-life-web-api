use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ExpenseSource {
    pub id: i64,
    pub name: String,
    pub expense: RecurringMoneyValue,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct RecurringMoneyValue {
    pub amount: i32,
    pub period: Period,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Period {
    pub kind: PeriodKind,
    pub every: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum PeriodKind {
    Month,
    Year,
}
