use chrono::NaiveDate;

use crate::domain::currency::Currency;

#[derive(Clone, Debug)]
pub struct CalculationContext {
    pub base_currency: Currency,
    pub as_of_date: NaiveDate,
}

impl CalculationContext {
    #[must_use]
    pub fn new(base_currency: Currency, as_of_date: NaiveDate) -> Self {
        CalculationContext {
            base_currency,
            as_of_date,
        }
    }
}
