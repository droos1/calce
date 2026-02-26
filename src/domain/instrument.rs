use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstrumentId(String);

impl InstrumentId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        InstrumentId(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstrumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
