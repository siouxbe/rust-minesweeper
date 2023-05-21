use super::*;
use crate::coordinations as cd;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

pub mod from_slave {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub enum MessageSentByClient {
        Join(JoinRequest),
        Click(Click),
    }

    impl From<MessageSentByClient> for MessageFromSlave {
        fn from(msg: MessageSentByClient) -> Self {
            match msg {
                MessageSentByClient::Join(j) => Self::Join(j.into()),
                MessageSentByClient::Click(c) => Self::Action(c.into()),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Click {
        pub session: data::SID,
        pub left: bool,
        pub coord: data::Coord,
    }

    impl From<Click> for ActionFromSlave {
        fn from(click: Click) -> Self {
            let Click {
                session,
                left,
                coord,
            } = click;
            Self {
                session: session.into(),
                coord: coord.into(),
                left,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct JoinRequest {
        pub name: String,
    }

    impl From<JoinRequest> for RequestFromSlave {
        fn from(r: JoinRequest) -> Self {
            let JoinRequest { name } = r;
            Self { name }
        }
    }
}

pub mod from_master {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Update {
        pub user: data::UID,
        pub session: data::SID,
        pub dimensions: data::Dimensions,
        pub status: data::Status,
        pub updates: data::Updates,
        pub users: data::Users,
    }

    impl From<Update> for UpdateFromMaster {
        fn from(update: Update) -> Self {
            let Update {
                user: _,
                session,
                dimensions,
                status,
                updates,
                users,
            } = update;
            let (stats, names) = users.into();
            let coords = dimensions.into();
            let client = UpdateFromMasterForClient {
                coords,
                updates: updates.into(),
            };
            let namer = UpdateFromMasterForNamer { names };
            let slave = GameUpdateFromMasterForSlave {
                coords,
                stats,
                session: session.into(),
                status: status.into(),
            };
            let slave = UpdateFromMasterForSlave { game: slave, namer };
            Self { client, slave }
        }
    }

    impl From<(UpdateFromMaster, SessionUserID)> for Update {
        fn from((update, uid): (UpdateFromMaster, SessionUserID)) -> Self {
            let UpdateFromMaster { client, slave } = update;
            let UpdateFromMasterForClient { coords, updates } = client;
            let UpdateFromMasterForSlave { game, namer } = slave;
            let GameUpdateFromMasterForSlave {
                coords: _,
                stats,
                session,
                status,
            } = game;
            let UpdateFromMasterForNamer { names } = namer;
            let users = (stats, names).into();
            Self {
                user: uid.into(),
                session: session.into(),
                dimensions: coords.into(),
                status: status.into(),
                updates: updates.into(),
                users,
            }
        }
    }
}

pub mod data {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[allow(clippy::upper_case_acronyms)]
    pub struct UID(u8);

    impl From<UID> for SessionUserID {
        fn from(uid: UID) -> Self {
            let UID(uid) = uid;
            Self::new(uid)
        }
    }

    impl From<SessionUserID> for UID {
        fn from(uid: SessionUserID) -> Self {
            let SessionUserID(uid) = uid;
            Self(uid)
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[allow(clippy::upper_case_acronyms)]
    pub struct SID(u8);

    impl From<SID> for SessionID {
        fn from(sid: SID) -> Self {
            let SID(sid) = sid;
            Self::new(sid)
        }
    }

    impl From<SessionID> for SID {
        fn from(sid: SessionID) -> Self {
            Self(sid.value())
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Dimensions {
        width: u32,
        height: u32,
    }

    impl From<Dimensions> for cd::Coordinations {
        fn from(dimensions: Dimensions) -> Self {
            let Dimensions { width, height } = dimensions;
            Self::from_width_and_height(width, height)
        }
    }

    impl From<cd::Coordinations> for Dimensions {
        fn from(coords: cd::Coordinations) -> Self {
            let width = coords.columns();
            let height = coords.rows();
            Self { width, height }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Coord {
        x: u32,
        y: u32,
    }

    impl From<Coord> for cd::Coord {
        fn from(coord: Coord) -> Self {
            let Coord { x, y } = coord;
            Self { x, y }
        }
    }

    impl From<cd::Coord> for Coord {
        fn from(coord: cd::Coord) -> Self {
            let cd::Coord { x, y } = coord;
            Self { x, y }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Hint(u8);

    impl From<Hint> for super::Hint {
        fn from(hint: Hint) -> Self {
            let Hint(hint) = hint;
            Self(hint)
        }
    }

    impl From<super::Hint> for Hint {
        fn from(hint: super::Hint) -> Self {
            let super::Hint(hint) = hint;
            Self(hint)
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Cell {
        Covered,
        Mine,
        HintBy(Hint, UID),
        FlaggedBy(UID),
        ExplodedBy(UID),
        FalseFlaggedBy(UID),
        QuestionMarked(UID),
    }

    impl From<Cell> for super::Cell {
        fn from(cell: Cell) -> Self {
            match cell {
                Cell::Covered => Self::Covered,
                Cell::Mine => Self::Mine,
                Cell::HintBy(hint, uid) => Self::HintBy(hint.into(), uid.into()),
                Cell::FlaggedBy(uid) => Self::FlaggedBy(uid.into()),
                Cell::ExplodedBy(uid) => Self::ExplodedBy(uid.into()),
                Cell::FalseFlaggedBy(uid) => Self::FalseFlaggedBy(uid.into()),
                Cell::QuestionMarked(uid) => Self::QuestionMarked(uid.into()),
            }
        }
    }

    impl From<super::Cell> for Cell {
        fn from(cell: super::Cell) -> Self {
            match cell {
                super::Cell::Covered => Self::Covered,
                super::Cell::Mine => Self::Mine,
                super::Cell::HintBy(hint, uid) => Self::HintBy(hint.into(), uid.into()),
                super::Cell::FlaggedBy(uid) => Self::FlaggedBy(uid.into()),
                super::Cell::ExplodedBy(uid) => Self::ExplodedBy(uid.into()),
                super::Cell::FalseFlaggedBy(uid) => Self::FalseFlaggedBy(uid.into()),
                super::Cell::QuestionMarked(uid) => Self::QuestionMarked(uid.into()),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Update {
        coord: Coord,
        cell: Cell,
    }

    impl From<Update> for CellUpdate {
        fn from(update: Update) -> Self {
            let Update { coord, cell } = update;
            Self {
                coord: coord.into(),
                cell: cell.into(),
            }
        }
    }

    impl From<CellUpdate> for Update {
        fn from(update: CellUpdate) -> Self {
            let CellUpdate { coord, cell } = update;
            Self {
                coord: coord.into(),
                cell: cell.into(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Stats {
        mines_left: i32,
        lives_left: u32,
    }

    impl From<Stats> for super::Stats {
        fn from(stats: Stats) -> Self {
            let Stats {
                mines_left,
                lives_left,
            } = stats;
            Self {
                mines_left,
                lives_left,
            }
        }
    }

    impl From<super::Stats> for Stats {
        fn from(stats: super::Stats) -> Self {
            let super::Stats {
                mines_left,
                lives_left,
            } = stats;
            Self {
                mines_left,
                lives_left,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Updates {
        cells: Vec<Update>,
        stats: Stats,
    }

    impl From<Vec<Update>> for CellUpdates {
        fn from(updates: Vec<Update>) -> Self {
            Self(updates.into_iter().map(|update| update.into()).collect())
        }
    }

    impl From<CellUpdates> for Vec<Update> {
        fn from(cells: CellUpdates) -> Self {
            let CellUpdates(cells) = cells;
            cells.into_iter().map(|c| c.into()).collect()
        }
    }

    impl From<Updates> for super::Updates {
        fn from(updates: Updates) -> Self {
            let Updates { cells, stats } = updates;
            Self {
                cells: cells.into(),
                stats: stats.into(),
            }
        }
    }

    impl From<super::Updates> for Updates {
        fn from(updates: super::Updates) -> Self {
            let super::Updates { cells, stats } = updates;
            Self {
                cells: cells.into(),
                stats: stats.into(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct User {
        id: UID,
        name: String,
        marked_correct: u32,
        marked_incorrect: u32,
        exploded: u32,
    }

    impl From<User> for ((SessionUserID, UserStat), UserName) {
        fn from(user: User) -> Self {
            let User {
                id,
                name,
                marked_correct,
                marked_incorrect,
                exploded,
            } = user;
            let uid = id.into();
            let stat = UserStat {
                marked_correct,
                marked_incorrect,
                exploded,
            };
            let username = UserName { uid, name };
            ((uid, stat), username)
        }
    }

    impl From<(SessionUserID, UserStat, UserName)> for User {
        fn from((uid, stat, name): (SessionUserID, UserStat, UserName)) -> Self {
            let id = uid.into();
            let UserName { uid: _, name } = name;
            let UserStat {
                marked_correct,
                marked_incorrect,
                exploded,
            } = stat;
            Self {
                id,
                name,
                marked_correct,
                marked_incorrect,
                exploded,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Users(Vec<User>);

    impl From<Users> for (UserStats, UserNames) {
        fn from(users: Users) -> Self {
            let Users(users) = users;
            let (stats, names): (HashMap<SessionUserID, UserStat>, Vec<UserName>) =
                users.into_iter().map(|user| user.into()).unzip();
            (UserStats(stats), UserNames(names))
        }
    }

    impl From<(UserStats, UserNames)> for Users {
        fn from((stats, names): (UserStats, UserNames)) -> Self {
            let UserStats(stats) = stats;
            let UserNames(names) = names;
            let users = names
                .into_iter()
                .map(|name| {
                    let uid = &name.uid;
                    let stat = stats
                        .iter()
                        .find_map(|(stat_uid, stat)| (stat_uid == uid).then_some(*stat))
                        .unwrap_or_default();
                    let user_data: (SessionUserID, UserStat, super::UserName) = (*uid, stat, name);
                    user_data.into()
                })
                .collect();
            Self(users)
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Status {
        Playing,
        Victory,
        Defeat,
    }

    impl From<Status> for super::Status {
        fn from(status: Status) -> Self {
            match status {
                Status::Playing => Self::Playing,
                Status::Victory => Self::Ended { success: true },
                Status::Defeat => Self::Ended { success: false },
            }
        }
    }

    impl From<super::Status> for Status {
        fn from(status: super::Status) -> Self {
            match status {
                super::Status::Playing => Self::Playing,
                super::Status::Ended { success: true } => Self::Victory,
                super::Status::Ended { success: false } => Self::Defeat,
            }
        }
    }
}
