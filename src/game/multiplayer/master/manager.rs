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
        let client = {
            let client = game::client::session::Session::new(coords, mines, lives);
            let client = sync::Mutex::new(client);
            sync::Arc::new(client)
        };
        let server = {
            let server = game::server::session::Session::new(coords, mines, lives);
            let server = server::Server::new(server, messenger.clone(), self.name.clone());
            let server = sync::Mutex::new(server);
            sync::Arc::new(server)
        };
        let slave_listener = MySlaveListener {
            server: sync::Arc::downgrade(&server),
            client: sync::Arc::downgrade(&client),
        };
        let messenger_thread = messenger_thread::spawn_read_messages_from_slaves_to_master_thread(
            sync::Arc::downgrade(&messenger),
            slave_listener,
        );
        MultiplayerSession(session::Session::new(client, server, messenger_thread))
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

struct MySlaveListener {
    client: sync::Weak<sync::Mutex<game::client::session::Session>>,
    server: sync::Weak<sync::Mutex<server::Server>>,
}

impl SlaveListener for MySlaveListener {
    fn on_request_to_join(&mut self, request: RequestFromSlave, addr: std::net::SocketAddr) {
        if let Some(server) = self.server.upgrade() {
            server
                .lock()
                .expect("Failed to lock multiplayer server session")
                .on_request_to_join(request, addr)
        }
    }

    fn on_action_from_slave(&mut self, action: ActionFromSlave, addr: std::net::SocketAddr) {
        if let Some(server) = self.server.upgrade() {
            let updates = server
                .lock()
                .expect("Failed to lock multiplayer server session")
                .on_action_from_slave(action, addr);
            if let Some(updates) = updates {
                if let Some(client) = self.client.upgrade() {
                    client
                        .lock()
                        .expect("Failed to lock multiplayer client session")
                        .on_updates(updates);
                }
            }
        }
    }
}
