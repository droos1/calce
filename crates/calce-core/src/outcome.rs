/// A calculation result that may include warnings about degraded data quality.
///
/// Functions return `Outcome<T>` when partial success is meaningful — e.g. a
/// portfolio with 50 positions where 1 price is missing should return 49 valued
/// positions plus a warning, not fail entirely.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Outcome<T> {
    pub value: T,
    pub warnings: Vec<Warning>,
}

impl<T> Outcome<T> {
    #[must_use]
    pub fn ok(value: T) -> Self {
        Outcome {
            value,
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_warnings(value: T, warnings: Vec<Warning>) -> Self {
        Outcome { value, warnings }
    }

    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Transform the value, keeping warnings.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Outcome<U> {
        Outcome {
            value: f(self.value),
            warnings: self.warnings,
        }
    }

    /// Merge warnings from another outcome.
    pub fn merge_warnings(&mut self, other: &Outcome<impl std::fmt::Debug>) {
        self.warnings.extend(other.warnings.iter().cloned());
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Warning {
    pub code: WarningCode,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WarningCode {
    /// Price not found for an instrument on a date
    MissingPrice,
    /// FX rate not found for a currency pair on a date
    MissingFxRate,
}

impl Warning {
    #[must_use]
    pub fn missing_price(message: impl Into<String>) -> Self {
        Warning {
            code: WarningCode::MissingPrice,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn missing_fx_rate(message: impl Into<String>) -> Self {
        Warning {
            code: WarningCode::MissingFxRate,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_has_no_warnings() {
        let outcome: Outcome<i32> = Outcome::ok(42);
        assert_eq!(outcome.value, 42);
        assert!(!outcome.has_warnings());
    }

    #[test]
    fn with_warnings_carries_them() {
        let outcome =
            Outcome::with_warnings(42, vec![Warning::missing_price("AAPL on 2025-01-15")]);
        assert!(outcome.has_warnings());
        assert_eq!(outcome.warnings.len(), 1);
        assert_eq!(outcome.warnings[0].code, WarningCode::MissingPrice);
    }

    #[test]
    fn map_preserves_warnings() {
        let outcome = Outcome::with_warnings(10, vec![Warning::missing_price("test")]);
        let doubled = outcome.map(|v| v * 2);
        assert_eq!(doubled.value, 20);
        assert_eq!(doubled.warnings.len(), 1);
    }
}
