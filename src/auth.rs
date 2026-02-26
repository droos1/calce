use crate::domain::user::UserId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

#[derive(Clone, Debug)]
pub struct SecurityContext {
    pub user_id: UserId,
    pub role: Role,
}

impl SecurityContext {
    #[must_use]
    pub fn new(user_id: UserId, role: Role) -> Self {
        SecurityContext { user_id, role }
    }

    #[must_use]
    pub fn system() -> Self {
        SecurityContext {
            user_id: UserId::new("system"),
            role: Role::Admin,
        }
    }

    #[must_use]
    pub fn can_access(&self, target: &UserId) -> bool {
        self.role == Role::Admin || self.user_id == *target
    }
}
