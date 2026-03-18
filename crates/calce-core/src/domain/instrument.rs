string_id!(InstrumentId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum InstrumentType {
    Stock,
    Bond,
    Etf,
    MutualFund,
    Certificate,
    Option,
    Warrant,
    StructuredProduct,
    Future,
    #[default]
    Other,
}

impl InstrumentType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stock => "stock",
            Self::Bond => "bond",
            Self::Etf => "etf",
            Self::MutualFund => "mutual_fund",
            Self::Certificate => "certificate",
            Self::Option => "option",
            Self::Warrant => "warrant",
            Self::StructuredProduct => "structured_product",
            Self::Future => "future",
            Self::Other => "other",
        }
    }

    /// Case-insensitive parse; unknown values map to `Other`.
    #[must_use]
    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "stock" => Self::Stock,
            "bond" => Self::Bond,
            "etf" => Self::Etf,
            "mutual_fund" | "mutualfund" => Self::MutualFund,
            "certificate" => Self::Certificate,
            "option" => Self::Option,
            "warrant" => Self::Warrant,
            "structured_product" | "structuredproduct" => Self::StructuredProduct,
            "future" => Self::Future,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for InstrumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
