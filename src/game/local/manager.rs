use super::*;

use crate::game;

pub struct LocalSessionManager {
    config: game::session::SessionConfig,
}

impl LocalSessionManager {
    pub fn new(config: game::session::SessionConfig) -> Self {
        Self { config }
    }
}

impl game::session::SessionManager for LocalSessionManager {
    type Session = session::Session;

    fn request_new_session(&self) -> Self::Session {
        session::Session::new(&self.config)
    }
}
