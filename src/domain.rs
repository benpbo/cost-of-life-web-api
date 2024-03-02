use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct ExpenseSource {
    pub id: i64,
    pub name: String,
    pub expense: RecurringMoneyValue,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct RecurringMoneyValue {
    pub amount: i32,
    pub period: Period,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Period {
    pub kind: PeriodKind,
    pub every: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum PeriodKind {
    Month,
    Year,
}
