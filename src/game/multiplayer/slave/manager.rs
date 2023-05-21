use super::*;

use std::sync;

pub struct Slave(pub std::net::SocketAddr);
pub struct Master(pub std::net::SocketAddr);

pub struct Manager {
    name: String,
    slave: std::net::SocketAddr,
    master: std::net::SocketAddr,
}

impl Manager {
    pub fn new(name: String, slave: Slave, master: Master) -> Self {
        let Slave(slave) = slave;
        let Master(master) = master;
        Self {
            name,
            slave,
            master,
        }
    }
}

impl game::session::SessionManager for Manager {
    type Session = MultiplayerSession;

    fn request_new_session(&self) -> Self::Session {
        let remote = std::net::UdpSocket::bind(self.slave).expect("Failed to bind local socket");
        remote
            .connect(self.master)
            .expect("Failed to connect to master server");
        let messenger = sync::Arc::new(Messenger::new(remote));
        let thread_messenger = sync::Arc::downgrade(&messenger);
        let mut buffer = MessengerBuffer::new();
        let UpdateFromMaster {
            client: client_initial,
            slave: slave_initial,
        } = messenger.request_to_join(&mut buffer, &self.name);
        let client = client::create_client_session_from_updates_from_master(client_initial);
        let server = server::Server::new(messenger, buffer, slave_initial);
        let session = {
            let session = session::Session::new(client, server);
            let session = sync::Mutex::new(session);
            sync::Arc::new(session)
        };
        let master_listener = MyMasterListener(sync::Arc::downgrade(&session));
        let _messenger_thread = messenger_thread::spawn_read_updates_from_master_to_slave_thread(
            thread_messenger,
            master_listener,
        );
        MultiplayerSession {
            session,
            _messenger_thread,
        }
    }
}

pub struct MultiplayerSession {
    session: sync::Arc<sync::Mutex<session::Session>>,
    _messenger_thread: MessengerThread,
}

impl game::session::Session for MultiplayerSession {
    fn snapshot<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(game::session::SessionSnapshot<'a>) -> R,
    {
        let Self {
            session,
            _messenger_thread: _,
        } = self;
        let session = &mut session.lock().expect("Failed to lock multiplayer session");
        session.snapshot(f)
    }
}

struct MyMasterListener(sync::Weak<sync::Mutex<session::Session>>);

impl MasterListener for MyMasterListener {
    fn on_updates_from_master_to_slave(&mut self, update: UpdateFromMaster) {
        let Self(session) = self;
        if let Some(session) = session.upgrade() {
            session
                .lock()
                .expect("Failed to lock multiplayer session")
                .on_updates_from_master_to_slave(update);
        }
    }
}
