use super::*;

use crate::game::session::Namer;

use std::cell::RefCell;
use std::rc::Rc;

/// Contains all data required to run a single-player game
pub struct Session {
    server: server::Server,
    client: ClientSessionCell,
    namer: LocalSessionNamer,
}

impl Session {
    pub fn new(config: &game::session::SessionConfig) -> Self {
        let game::session::SessionConfig {
            coords,
            mines,
            lives,
        } = *config;
        let client = game::client::session::Session::new(coords, mines, lives);
        let client = ClientSessionCell::new(client);
        let server = game::server::session::Session::new(coords, mines, lives);
        let local_updates_listener = client.clone();
        let server = server::Server::new(server, Box::new(local_updates_listener), SessionUserID::new(1));
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
            namer: Box::new(self.namer.clone()),
            field_provider: &self.client,
            local_player_listener: &mut self.server,
        })
    }
}

#[derive(Clone)]
struct LocalSessionNamer;

impl Namer for LocalSessionNamer {
    fn name(&self, _uid: SessionUserID) -> &str {
        "Single Player"
    }
}

#[derive(Clone)]
struct ClientSessionCell(Rc<RefCell<game::client::session::Session>>);

impl ClientSessionCell {
    pub fn new(session: game::client::session::Session) -> Self {
        Self(Rc::new(RefCell::new(session)))
    }

    pub fn stats(&self) -> Stats {
        let Self(session) = self;
        session.borrow().stats()
    }
}

impl UpdatesListener for ClientSessionCell {
    fn on_updates(&mut self, updates: Updates) {
        let Self(session) = self;
        session.borrow_mut().on_updates(updates)
    }
}

impl FieldProvider for ClientSessionCell {
    fn get_cell(&self, coord: &Coord) -> Cell {
        let Self(session) = self;
        session.borrow().get_cell(coord)
    }
}
