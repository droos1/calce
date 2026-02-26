use rust_decimal::Decimal;

/// Positive = long, negative = short.
#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Add)]
pub struct Quantity(Decimal);

impl Quantity {
    #[must_use]
    pub fn new(value: Decimal) -> Self {
        Quantity(value)
    }

    #[must_use]
    pub fn value(&self) -> Decimal {
        self.0
    }
}
