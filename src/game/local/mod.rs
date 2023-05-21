mod manager;
mod server;
mod session;

use super::*;
use crate::game;

pub fn create_manager(cfg: game::session::SessionConfig) -> manager::LocalSessionManager {
    manager::LocalSessionManager::new(cfg)
}
