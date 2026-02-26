use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
pub struct Price(Decimal);

impl Price {
    #[must_use]
    pub fn new(value: Decimal) -> Self {
        Price(value)
    }

    #[must_use]
    pub fn value(&self) -> Decimal {
        self.0
    }
}
