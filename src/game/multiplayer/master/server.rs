use super::*;

use crate::MAX_PLAYERS;

pub struct Server {
    core: Core,
    players: players::Players,
}

impl Server {
    pub fn new(
        server: game::server::session::Session,
        messenger: std::sync::Arc<Messenger>,
        name: String,
    ) -> Self {
        let my_uid = SessionUserID(1);
        let core = Core::new(server, messenger, my_uid);
        let players = players::Players::new(my_uid, name);
        Self { core, players }
    }

    pub fn status(&self) -> Status {
        self.core.server.status()
    }

    pub fn coords(&self) -> Coordinations {
        self.core.server.coords()
    }

    pub fn user_stats(&self) -> UserStats {
        self.core.server.user_stats()
    }

    pub fn servitors(&mut self) -> (MasterLocalUpdatesListener<'_>, &dyn game::session::Namer) {
        let local_player_listener = MasterLocalUpdatesListener {
            core: &mut self.core,
            players: &self.players,
        };
        let namer = &self.players;
        (local_player_listener, namer)
    }

    #[allow(dead_code)]
    pub fn on_request_to_join(&mut self, request: RequestFromSlave, addr: std::net::SocketAddr) {
        let RequestFromSlave { name } = request;
        if let Some(player_uid) = self.players.try_add(name, addr) {
            let reply = create_update_message(&self.core, &self.players, self.core.server.all());
            self.core
                .messenger
                .send_updates_from_master(&mut self.core.messenger_buffer, addr, player_uid, reply)
                .unwrap();
        }
    }

    #[allow(dead_code)]
    pub fn on_action_from_slave(
        &mut self,
        action: ActionFromSlave,
        addr: std::net::SocketAddr,
    ) -> Option<Updates> {
        let ActionFromSlave {
            session,
            coord,
            left,
        } = action;
        if session != self.core.sessionid {
            return None;
        }
        let uid = self.players.get_uid(&addr)?;
        let mut player_listener = MasterLocalUpdatesListener {
            core: &mut self.core,
            players: &self.players,
        };
        Some(if left {
            player_listener.on_left_click_id(uid, &coord)
        } else {
            player_listener.on_right_click_id(uid, &coord)
        })
    }
}

fn create_update_message(
    core: &Core,
    players: &players::Players,
    cells: CellUpdates,
) -> UpdateFromMaster {
    let updates = Updates {
        cells,
        stats: core.server.stats(),
    };
    let client = UpdateFromMasterForClient {
        coords: core.server.coords(),
        updates,
    };
    let game = GameUpdateFromMasterForSlave {
        coords: core.server.coords(),
        session: core.sessionid,
        status: core.server.status(),
        stats: core.server.user_stats(),
    };
    let namer = UpdateFromMasterForNamer {
        names: players.names(),
    };
    let slave = UpdateFromMasterForSlave { game, namer };
    UpdateFromMaster { client, slave }
}

struct Core {
    server: game::server::session::Session,
    my_uid: SessionUserID,
    messenger: std::sync::Arc<Messenger>,
    messenger_buffer: MessengerBuffer,
    sessionid: SessionID,
}

impl Core {
    fn new(
        server: game::server::session::Session,
        messenger: std::sync::Arc<Messenger>,
        my_uid: SessionUserID,
    ) -> Self {
        let messenger_buffer = MessengerBuffer::new();
        let sessionid = {
            let mut id = 0u8;
            use rand::RngCore;
            rand::thread_rng().fill_bytes(std::slice::from_mut(&mut id));
            SessionID::new(id)
        };
        Self {
            server,
            my_uid,
            messenger,
            messenger_buffer,
            sessionid,
        }
    }
}

pub struct MasterLocalUpdatesListener<'a> {
    core: &'a mut Core,
    players: &'a players::Players,
}

impl MasterLocalUpdatesListener<'_> {
    fn on_left_click_id(&mut self, uid: SessionUserID, coord: &Coord) -> Updates {
        let updates = self.core.server.uncover(coord, uid);
        self.on_click(updates)
    }

    fn on_right_click_id(&mut self, uid: SessionUserID, coord: &Coord) -> Updates {
        let updates = self.core.server.toggle_mark(coord, uid);
        self.on_click(updates)
    }

    fn on_click(&mut self, cells: CellUpdates) -> Updates {
        let stats = self.core.server.stats();
        let updates = Updates {
            cells: cells.clone(),
            stats,
        };
        let update = create_update_message(self.core, self.players, cells);
        for peer in self.players.peers() {
            self.core
                .messenger
                .send_updates_from_master(
                    &mut self.core.messenger_buffer,
                    peer.addr,
                    peer.uid,
                    update.clone(),
                )
                .unwrap();
        }
        updates
    }
}

impl MPLocalPlayerListener for MasterLocalUpdatesListener<'_> {
    fn on_left_click(&mut self, coord: &Coord) -> Option<Updates> {
        Some(self.on_left_click_id(self.core.my_uid, coord))
    }

    fn on_right_click(&mut self, coord: &Coord) -> Option<Updates> {
        Some(self.on_right_click_id(self.core.my_uid, coord))
    }
}

mod players {

    use super::*;

    #[derive(Debug)]
    pub struct Player {
        pub uid: SessionUserID,
        pub name: String,
        pub addr: std::net::SocketAddr,
    }

    #[derive(Debug)]
    pub struct Players {
        my_uid: SessionUserID,
        my_name: String,
        slaves: Vec<Player>,
    }

    impl Players {
        pub fn new(my_uid: SessionUserID, my_name: String) -> Self {
            let slaves = Vec::new();
            Self {
                my_uid,
                my_name,
                slaves,
            }
        }

        pub fn try_add(
            &mut self,
            name: String,
            addr: std::net::SocketAddr,
        ) -> Option<SessionUserID> {
            let count = 1 + self.slaves.len();
            let space_available = count < MAX_PLAYERS.into();
            let addr_in_use = self.slaves.iter().any(|player| player.addr == addr);
            (space_available && !addr_in_use).then(|| {
                let uid = next_uid(&self.slaves);
                let player = Player { uid, name, addr };
                self.slaves.push(player);
                uid
            })
        }

        pub fn names(&self) -> UserNames {
            todo!()
        }

        pub fn get_uid(&self, peer: &std::net::SocketAddr) -> Option<SessionUserID> {
            self.slaves
                .iter()
                .find_map(|Player { uid, name: _, addr }| (addr == peer).then_some(uid))
                .copied()
        }

        pub fn peers(&self) -> impl Iterator<Item = &Player> {
            self.slaves.iter()
        }
    }

    impl game::session::Namer for Players {
        fn name(&self, uid: SessionUserID) -> &str {
            let Players {
                my_uid,
                my_name,
                slaves,
            } = self;
            if *my_uid == uid {
                return my_name;
            }
            let id = uid;
            slaves
                .iter()
                .find_map(|Player { uid, name, addr: _ }| (id == *uid).then_some(name))
                .expect("Master can't find that userid")
        }
    }

    fn next_uid(slaves: &[Player]) -> SessionUserID {
        let uid = 2 + slaves.len();
        SessionUserID(uid.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_addr(port: u16) -> std::net::SocketAddr {
        std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::UNSPECIFIED,
            port,
        ))
    }

    fn create_players() -> players::Players {
        let mut p = players::Players::new(SessionUserID(3), "Cedric".into());
        p.try_add("Alice".into(), make_addr(1)).unwrap();
        p.try_add("Bob".into(), make_addr(2)).unwrap();
        p
    }

    #[test]
    fn players_get_names() {
        let UserNames(names) = create_players().names();
        assert_eq!(names.len(), 3);
        let mut names = names.iter();
        assert_eq!(names.next().unwrap().name, "Cedric");
        assert_eq!(names.next().unwrap().name, "Alice");
        assert_eq!(names.next().unwrap().name, "Bob");
    }
}
