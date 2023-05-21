use super::*;

pub struct Server {
    core: Core,
    namer: Namer,
}

impl Server {
    pub fn new(
        messenger: std::sync::Arc<Messenger>,
        buffer: MessengerBuffer,
        initial: UpdateFromMasterForSlave,
    ) -> Self {
        let UpdateFromMasterForSlave {
            game: initial_game,
            namer: initial_namer,
        } = initial;
        let namer = Namer {
            latest: initial_namer,
        };
        let core = Core {
            messenger,
            buffer,
            latest: initial_game,
        };
        Self { core, namer }
    }

    pub fn status(&self) -> Status {
        self.core.latest.status
    }

    pub fn coords(&self) -> Coordinations {
        self.core.latest.coords
    }

    pub fn user_stats(&self) -> UserStats {
        self.core.latest.stats.clone()
    }

    pub fn servitors(&mut self) -> (&mut dyn LocalPlayerListener, &dyn game::session::Namer) {
        (&mut self.core, &self.namer)
    }

    pub fn on_updates_from_master(&mut self, update: UpdateFromMasterForSlave) {
        let UpdateFromMasterForSlave { game, namer } = update;
        self.core.latest = game;
        self.namer.latest = namer;
    }
}

struct Core {
    messenger: std::sync::Arc<Messenger>,
    buffer: MessengerBuffer,
    latest: GameUpdateFromMasterForSlave,
}

impl LocalPlayerListener for Core {
    fn on_left_click(&mut self, coord: &Coord) {
        let sessionid = self.latest.session;
        self.messenger
            .left_click(&mut self.buffer, sessionid, coord)
    }

    fn on_right_click(&mut self, coord: &Coord) {
        let sessionid = self.latest.session;
        self.messenger
            .right_click(&mut self.buffer, sessionid, coord)
    }
}

struct Namer {
    latest: UpdateFromMasterForNamer,
}

impl game::session::Namer for Namer {
    fn name(&self, uid: SessionUserID) -> &str {
        let id = uid;
        let UserNames(usernames) = &self.latest.names;
        usernames
            .iter()
            .find_map(|UserName { uid, name }| (id == *uid).then_some(name))
            .expect("Master hasn't sent that userid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::game::session::Namer;

    const ALICE_UID: game::SessionUserID = game::SessionUserID(7);
    const BOB_UID: game::SessionUserID = game::SessionUserID(21);

    fn create_namer() -> super::Namer {
        let alice = UserName {
            uid: ALICE_UID,
            name: "Alice".into(),
        };
        let bob = UserName {
            uid: BOB_UID,
            name: "Bob".into(),
        };
        let latest = UpdateFromMasterForNamer {
            names: UserNames(vec![alice, bob]),
        };
        super::Namer { latest }
    }

    #[test]
    fn namer_existing_alice() {
        let namer = create_namer();
        assert_eq!(namer.name(ALICE_UID), "Alice");
    }

    #[test]
    fn namer_existing_bob() {
        let namer = create_namer();
        assert_eq!(namer.name(BOB_UID), "Bob");
    }

    #[test]
    #[should_panic]
    fn namer_nonexisting() {
        let namer = create_namer();
        let _ = namer.name(game::SessionUserID(255));
    }
}
