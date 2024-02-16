use serde::{Deserialize, Serialize};

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
