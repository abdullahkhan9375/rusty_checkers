use std::collections::HashSet;

pub struct Users {
    registered_users: HashSet<String>,
    logged_in_users: HashSet<String>,
}

#[derive(Debug)]
pub enum LoginResult {
    Success,
    UserNotFound,
    UserAlreadyLoggedIn,
}

#[derive(Debug)]
pub enum LogoutResult {
    Success,
    UserNotFound,
    UserNotLoggedIn,
}

impl Users {
    pub fn new() -> Self {
        // TODO: From database
        let registered_users = ["abdullah", "ben"]
            .into_iter()
            .map(str::to_string)
            .collect();

        Self {
            registered_users,
            logged_in_users: Default::default(),
        }
    }

    pub fn login_user(&mut self, username: &str) -> LoginResult {
        if !self.registered_users.contains(username) {
            return LoginResult::UserNotFound;
        }

        if self.logged_in_users.contains(username) {
            return LoginResult::UserAlreadyLoggedIn;
        }

        self.logged_in_users.insert(username.to_string());
        LoginResult::Success
    }

    pub fn logout_user(&mut self, username: &str) -> LogoutResult {
        if !self.registered_users.contains(username) {
            return LogoutResult::UserNotFound;
        }

        if !self.logged_in_users.contains(username) {
            return LogoutResult::UserNotLoggedIn;
        }

        self.logged_in_users.remove(username);
        LogoutResult::Success
    }

    pub fn is_logged_in(&self, username: &str) -> bool {
        self.logged_in_users.contains(username)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_login_logout() {
        let mut users = Users::new();
        assert!(matches!(users.login_user("joe"), LoginResult::UserNotFound));

        assert!(!users.is_logged_in("ben"));
        assert!(matches!(users.login_user("ben"), LoginResult::Success));
        assert!(users.is_logged_in("ben"));
        assert!(matches!(users.login_user("ben"), LoginResult::UserAlreadyLoggedIn));

        assert!(matches!(users.logout_user("joe"), LogoutResult::UserNotFound));
        assert!(matches!(users.logout_user("ben"), LogoutResult::Success));
        assert!(matches!(users.logout_user("ben"), LogoutResult::UserNotLoggedIn));
    }
}
