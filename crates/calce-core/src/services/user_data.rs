use std::collections::HashMap;

use crate::auth::SecurityContext;
use crate::domain::trade::Trade;
use crate::domain::user::UserId;
use crate::error::{CalceError, CalceResult};

pub trait UserDataService {
    /// # Errors
    ///
    /// Returns `Unauthorized` if the security context lacks access.
    /// Returns `NoTradesFound` if the user has no trades.
    fn get_trades(
        &self,
        ctx: &SecurityContext,
        user_id: &UserId,
    ) -> CalceResult<Vec<Trade>>;
}

/// In-memory implementation for testing.
#[derive(Default)]
pub struct InMemoryUserDataService {
    trades: HashMap<UserId, Vec<Trade>>,
}

impl InMemoryUserDataService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trade(&mut self, trade: Trade) {
        self.trades
            .entry(trade.user_id.clone())
            .or_default()
            .push(trade);
    }
}

impl UserDataService for InMemoryUserDataService {
    fn get_trades(
        &self,
        ctx: &SecurityContext,
        user_id: &UserId,
    ) -> CalceResult<Vec<Trade>> {
        if !ctx.can_access(user_id) {
            return Err(CalceError::Unauthorized {
                requester: ctx.user_id.clone(),
                target: user_id.clone(),
            });
        }
        self.trades
            .get(user_id)
            .cloned()
            .ok_or_else(|| CalceError::NoTradesFound(user_id.clone()))
    }
}
