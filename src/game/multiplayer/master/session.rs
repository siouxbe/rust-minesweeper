use super::*;

type Client = std::sync::Arc<std::sync::Mutex<game::client::session::Session>>;
type Server = std::sync::Arc<std::sync::Mutex<server::Server>>;

pub struct Session {
    client: Client,
    server: Server,
    _messenger_thread: MessengerThread,
}

impl Session {
    pub fn new(client: Client, server: Server, _messenger_thread: MessengerThread) -> Self {
        Self {
            client,
            server,
            _messenger_thread,
        }
    }
}

impl game::session::Session for Session {
    fn snapshot<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(game::session::SessionSnapshot<'a>) -> R,
    {
        let mut server = self
            .server
            .lock()
            .expect("Failed to lock multiplayer server session");
        let mut client = self
            .client
            .lock()
            .expect("Failed to lock multiplayer client session");
        let user_stats = server.user_stats();
        let status = server.status();
        let coords = server.coords();
        let stats = client.stats();
        let field_provider: &game::client::session::Session = &client;
        let (mut local_player_listener, namer) = server.servitors();
        let mut local_player_listener =
            record_updates::RecordUpdates::new(&mut local_player_listener);
        let r = f(game::session::SessionSnapshot {
            user_stats,
            status,
            coords,
            stats,
            namer,
            field_provider,
            local_player_listener: &mut local_player_listener,
        });
        if let Some(updates) = local_player_listener.updates() {
            client.on_updates(updates);
        }
        r
    }
}

mod record_updates {
    use super::*;

    pub struct RecordUpdates<'a, L> {
        record: Option<Updates>,
        local_player_listener: &'a mut L,
    }

    impl<'a, L> RecordUpdates<'a, L> {
        pub fn new(local_player_listener: &'a mut L) -> Self {
            Self {
                record: None,
                local_player_listener,
            }
        }

        pub fn updates(self) -> Option<Updates> {
            self.record
        }
    }

    impl<L> game::LocalPlayerListener for RecordUpdates<'_, L>
    where
        L: MPLocalPlayerListener,
    {
        fn on_left_click(&mut self, coord: &Coord) {
            let updates = self.local_player_listener.on_left_click(coord);
            aggregate(&mut self.record, updates)
        }

        fn on_right_click(&mut self, coord: &Coord) {
            let updates = self.local_player_listener.on_right_click(coord);
            aggregate(&mut self.record, updates)
        }
    }

    fn aggregate(record: &mut Option<Updates>, new: Option<Updates>) {
        *record = match (record.take(), new) {
            (record, None) => record,
            (None, new) => new,
            (Some(record), Some(new)) => Some(concatenate(record, new)),
        };
    }

    fn concatenate(record: Updates, new: Updates) -> Updates {
        let Updates {
            cells: CellUpdates(mut record),
            stats: _,
        } = record;
        let Updates {
            cells: CellUpdates(new),
            stats,
        } = new;
        record.extend(new);
        let cells = CellUpdates(record);
        Updates { cells, stats }
    }
}
