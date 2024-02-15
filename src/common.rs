use serde::Serialize;

#[derive(Serialize)]
pub struct RecurringMoneyValue {
    pub amount: i32,
    pub period: Period,
}

#[derive(Serialize)]
pub enum Period {
    Month,
    Year,
}
