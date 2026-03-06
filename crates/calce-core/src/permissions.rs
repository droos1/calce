//! Access control checks.
//!
//! Centralizes all "can user X do Y" decisions in one reviewable module.
//! Both the API edge and the data layer should call these functions.

use crate::auth::{Role, SecurityContext};
use crate::domain::user::UserId;

/// Can the authenticated user access the target user's data?
///
/// Rules:
/// - Admin can access any user's data
/// - A regular user can only access their own data
#[must_use]
pub fn can_access_user_data(ctx: &SecurityContext, target: &UserId) -> bool {
    ctx.role == Role::Admin || ctx.user_id == *target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_can_access_own_data() {
        let alice = UserId::new("alice");
        let ctx = SecurityContext::new(alice.clone(), Role::User);
        assert!(can_access_user_data(&ctx, &alice));
    }

    #[test]
    fn user_cannot_access_other_data() {
        let alice = UserId::new("alice");
        let bob = UserId::new("bob");
        let ctx = SecurityContext::new(bob, Role::User);
        assert!(!can_access_user_data(&ctx, &alice));
    }

    #[test]
    fn admin_can_access_any_data() {
        let alice = UserId::new("alice");
        let ctx = SecurityContext::system();
        assert!(can_access_user_data(&ctx, &alice));
    }
}
