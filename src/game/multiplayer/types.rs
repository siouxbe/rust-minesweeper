use super::*;

#[derive(Clone, Debug)]
pub struct UpdateFromMasterForClient {
    pub coords: Coordinations,
    pub updates: Updates,
}

#[derive(Clone, Debug)]
pub struct UserName {
    pub uid: SessionUserID,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct UserNames(pub Vec<UserName>);

#[derive(Clone, Debug)]
pub struct UpdateFromMasterForNamer {
    pub names: UserNames,
}

#[derive(Clone, Debug)]
pub struct GameUpdateFromMasterForSlave {
    pub coords: Coordinations,
    pub session: SessionID,
    pub status: Status,
    pub stats: UserStats,
}

#[derive(Clone, Debug)]
pub struct UpdateFromMasterForSlave {
    pub game: GameUpdateFromMasterForSlave,
    pub namer: UpdateFromMasterForNamer,
}

#[derive(Clone, Debug)]
pub struct UpdateFromMaster {
    pub client: UpdateFromMasterForClient,
    pub slave: UpdateFromMasterForSlave,
}

#[derive(Debug)]
pub struct ActionFromSlave {
    pub session: SessionID,
    pub coord: Coord,
    pub left: bool,
}

#[derive(Debug)]
pub struct RequestFromSlave {
    pub name: String,
}

pub enum MessageFromSlave {
    Join(RequestFromSlave),
    Action(ActionFromSlave),
}
