use super::*;

use std::sync;

pub struct Manager {
    name: String,
    master: std::net::SocketAddr,
    config: game::session::SessionConfig,
}

impl Manager {
    pub fn new(
        name: String,
        master: std::net::SocketAddr,
        config: game::session::SessionConfig,
    ) -> Self {
        Self {
            name,
            master,
            config,
        }
    }
}

impl game::session::SessionManager for Manager {
    type Session = MultiplayerSession;

    fn request_new_session(&self) -> Self::Session {
        let remote =
            std::net::UdpSocket::bind(self.master).expect("Failed to connect to master server");
        let messenger = sync::Arc::new(Messenger::new(remote));
        let game::session::SessionConfig {
            coords,
            mines,
            lives,
        } = self.config;
        let client = game::client::session::Session::new(coords, mines, lives);
        let server = {
            let server = game::server::session::Session::new(coords, mines, lives);
            server::Server::new(server, messenger, self.name.clone())
        };
        MultiplayerSession(session::Session::new(client, server))
    }
}

pub struct MultiplayerSession(session::Session);

impl game::session::Session for MultiplayerSession {
    fn snapshot<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(game::session::SessionSnapshot<'a>) -> R,
    {
        let Self(session) = self;
        session.snapshot(f)
    }
}
