use super::*;

use crate::game::session::Namer;

/// Contains all data required to run a single-player game
pub struct Session {
    server: server::Server<game::client::session::Session>,
    client: game::client::session::Session,
    namer: LocalSessionNamer,
}

impl Session {
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    #[allow(clippy::diverging_sub_expression)]
    pub fn new(config: &game::session::SessionConfig) -> Self {
        let game::session::SessionConfig {
            coords,
            mines,
            lives,
        } = *config;
        let client = game::client::session::Session::new(coords, mines, lives);
        let server = game::server::session::Session::new(coords, mines, lives);
        let local_updates_listener = todo!();
        let server = server::Server::new(server, local_updates_listener, SessionUserID::new(1));
        let namer = LocalSessionNamer;
        Self {
            server,
            client,
            namer,
        }
    }
}

impl game::session::Session for Session {
    fn snapshot<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(game::session::SessionSnapshot<'a>) -> R,
    {
        let user_stats = self.server.user_stats();
        let status = self.server.status();
        let coords = self.server.coords();
        f(game::session::SessionSnapshot {
            stats: self.client.stats(),
            user_stats,
            status,
            coords,
            namer: &self.namer,
            field_provider: &self.client,
            local_player_listener: &mut self.server,
        })
    }
}

struct LocalSessionNamer;

impl Namer for LocalSessionNamer {
    fn name(&self, _uid: SessionUserID) -> &str {
        "Single Player"
    }
}
