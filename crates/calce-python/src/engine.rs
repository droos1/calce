use chrono::NaiveDate;
use pyo3::prelude::*;

use calce_core::auth::{Role, SecurityContext};
use calce_core::calc::aggregation;
use calce_core::calc::market_value;
use calce_core::calc::volatility;
use calce_core::context::CalculationContext;
use calce_core::domain::instrument::InstrumentId;
use calce_core::domain::user::UserId;
use calce_core::reports::portfolio;
use calce_core::services::user_data::UserDataService;

use crate::domain::Currency;
use crate::errors::calce_err_to_py;
use crate::results::{MarketValueResult, PortfolioReport, VolatilityResult};
use crate::services::{MarketData, UserData};

#[pyclass]
pub struct CalcEngine {
    ctx: CalculationContext,
    security_ctx: SecurityContext,
    user_id: UserId,
    market_data: Py<MarketData>,
    user_data: Py<UserData>,
}

#[pymethods]
impl CalcEngine {
    #[new]
    #[pyo3(signature = (base_currency, as_of_date, user_id, market_data, user_data, role = "user"))]
    fn new(
        py: Python<'_>,
        base_currency: &Currency,
        as_of_date: NaiveDate,
        user_id: &str,
        market_data: Py<MarketData>,
        user_data: Py<UserData>,
        role: &str,
    ) -> PyResult<Self> {
        let role = match role {
            "admin" => Role::Admin,
            "user" => Role::User,
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "role must be \"user\" or \"admin\", got \"{other}\""
                )));
            }
        };
        let uid = UserId::new(user_id);
        market_data.borrow_mut(py).inner.freeze();
        Ok(CalcEngine {
            ctx: CalculationContext::new(base_currency.inner, as_of_date),
            security_ctx: SecurityContext::new(uid.clone(), role),
            user_id: uid,
            market_data,
            user_data,
        })
    }

    fn market_value(&self, py: Python<'_>) -> PyResult<MarketValueResult> {
        let md = self.market_data.borrow(py);
        let ud = self.user_data.borrow(py);
        let trades = ud
            .inner
            .get_trades(&self.security_ctx, &self.user_id)
            .map_err(calce_err_to_py)?;
        let positions = aggregation::aggregate_positions(&trades, self.ctx.as_of_date)
            .map_err(calce_err_to_py)?;
        // TODO: surface warnings to Python
        market_value::value_positions(&positions, &self.ctx, &md.inner)
            .map(|outcome| MarketValueResult {
                inner: outcome.value,
            })
            .map_err(calce_err_to_py)
    }

    fn portfolio_report(&self, py: Python<'_>) -> PyResult<PortfolioReport> {
        let md = self.market_data.borrow(py);
        let ud = self.user_data.borrow(py);
        let trades = ud
            .inner
            .get_trades(&self.security_ctx, &self.user_id)
            .map_err(calce_err_to_py)?;
        // TODO: surface warnings to Python
        portfolio::portfolio_report(&trades, &self.ctx, &md.inner)
            .map(|outcome| PortfolioReport {
                inner: outcome.value,
            })
            .map_err(calce_err_to_py)
    }

    /// Compute historical realized volatility for an instrument.
    ///
    /// Args:
    ///     instrument_id: Instrument ticker/identifier.
    ///     lookback_days: Calendar days of history (default 1095 = 3 years).
    #[pyo3(signature = (instrument_id, lookback_days = 1095))]
    fn volatility(
        &self,
        py: Python<'_>,
        instrument_id: &str,
        lookback_days: u32,
    ) -> PyResult<VolatilityResult> {
        let md = self.market_data.borrow(py);
        let instrument = InstrumentId::new(instrument_id);
        volatility::calculate_volatility(&instrument, self.ctx.as_of_date, lookback_days, &md.inner)
            .map(|r| VolatilityResult { inner: r })
            .map_err(calce_err_to_py)
    }
}
