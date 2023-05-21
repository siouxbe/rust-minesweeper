use super::*;

/// Contains all data required to run a single-player game
pub struct Server {
    server: game::server::session::Session,
    local_updates_listener: Box<dyn UpdatesListener>,
    uid: SessionUserID,
}

impl<L> Server<L>
where
    L: UpdatesListener,
{
    pub fn new(
        server: game::server::session::Session,
        local_updates_listener: L,
        uid: SessionUserID,
    ) -> Self {
        Self {
            server,
            local_updates_listener,
            uid,
        }
    }

    pub fn status(&self) -> Status {
        self.server.status()
    }

    pub fn user_stats(&self) -> UserStats {
        self.server.user_stats()
    }

    pub fn coords(&self) -> Coordinations {
        self.server.coords()
    }

    fn on_click(&mut self, cells: CellUpdates) {
        let stats = self.server.stats();
        let updates = Updates { cells, stats };
        self.local_updates_listener.on_updates(updates)
    }
}

impl<L> LocalPlayerListener for Server<L>
where
    L: UpdatesListener,
{
    fn on_left_click(&mut self, coord: &Coord) {
        let updates = self.server.uncover(coord, self.uid);
        self.on_click(updates)
    }

    fn on_right_click(&mut self, coord: &Coord) {
        let updates = self.server.toggle_mark(coord, self.uid);
        self.on_click(updates)
    }
}
