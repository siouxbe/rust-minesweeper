use super::*;

pub struct Session {
    client: game::client::session::Session,
    server: server::Server,
}

impl Session {
    pub fn new(client: game::client::session::Session, server: server::Server) -> Self {
        Self { client, server }
    }

    pub fn on_updates_from_master_to_slave(&mut self, update: UpdateFromMaster) {
        let UpdateFromMaster {
            client: client_update,
            slave: slave_update,
        } = update;
        self.server.on_updates_from_master(slave_update);
        let UpdateFromMasterForClient { coords: _, updates } = client_update;
        self.client.on_updates(updates);
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
        let stats = self.client.stats();
        let field_provider = &self.client;
        let (local_player_listener, namer) = self.server.servitors();
        f(game::session::SessionSnapshot {
            user_stats,
            status,
            coords,
            stats,
            namer: Box::new(NamerWrapper(namer)),
            field_provider,
            local_player_listener,
        })
    }
}
