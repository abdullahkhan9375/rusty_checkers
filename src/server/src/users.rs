use std::collections::{HashSet, HashMap};
use tokio::sync::mpsc;
use messaging::ServerMessage;

pub struct Users {
    registered_users: HashSet<String>,
    logged_in_users: HashMap<String, mpsc::Sender<ServerMessage>>,
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

    pub fn login_user(&mut self, username: &str, tx: mpsc::Sender<ServerMessage>) -> LoginResult {
        if !self.registered_users.contains(username) {
            return LoginResult::UserNotFound;
        }

        if self.logged_in_users.contains_key(username) {
            return LoginResult::UserAlreadyLoggedIn;
        }

        self.logged_in_users.insert(username.to_string(), tx);
        LoginResult::Success
    }

    pub fn logout_user(&mut self, username: &str) -> LogoutResult {
        if !self.registered_users.contains(username) {
            return LogoutResult::UserNotFound;
        }

        if !self.logged_in_users.contains_key(username) {
            return LogoutResult::UserNotLoggedIn;
        }

        self.logged_in_users.remove(username);
        LogoutResult::Success
    }

    pub fn is_logged_in(&self, username: &str) -> bool {
        self.logged_in_users.contains_key(username)
    }

    pub fn get_user_tx(&self, username: &str) -> Option<mpsc::Sender<ServerMessage>> {
        self.logged_in_users.get(username).cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_login_logout() {
        let (tx, _) = mpsc::channel(10);
        let mut users = Users::new();
        assert!(matches!(users.login_user("joe", tx.clone()), LoginResult::UserNotFound));

        assert!(!users.is_logged_in("ben"));
        assert!(matches!(users.login_user("ben", tx.clone()), LoginResult::Success));
        assert!(users.is_logged_in("ben"));
        assert!(matches!(users.login_user("ben", tx.clone()), LoginResult::UserAlreadyLoggedIn));

        assert!(matches!(users.logout_user("joe"), LogoutResult::UserNotFound));
        assert!(matches!(users.logout_user("ben"), LogoutResult::Success));
        assert!(matches!(users.logout_user("ben"), LogoutResult::UserNotLoggedIn));
    }
}
