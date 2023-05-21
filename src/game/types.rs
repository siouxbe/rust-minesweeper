use super::*;

#[allow(dead_code)]
pub const SYSTEM_UID: SessionUserID = SessionUserID(0);

pub trait UpdatesListener {
    fn on_updates(&mut self, updates: Updates);
}

#[must_use]
#[derive(Debug, Clone)]
pub struct Updates {
    pub cells: CellUpdates,
    pub stats: Stats,
}

#[must_use]
#[derive(Debug)]
pub struct Update {
    pub cell: CellUpdate,
    pub stats: Stats,
}

#[must_use]
#[derive(Debug, Clone)]
pub struct CellUpdate {
    pub cell: Cell,
    pub coord: Coord,
}

#[must_use]
#[derive(Default, Debug, Clone)]
pub struct CellUpdates(pub Vec<CellUpdate>);

impl CellUpdates {
    pub fn one(update: CellUpdate) -> Self {
        Self(vec![update])
    }

    pub fn none() -> Self {
        Self(Vec::new())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SessionID(u8);

impl SessionID {
    pub fn new(sid: u8) -> Self {
        Self(sid)
    }

    pub fn value(&self) -> u8 {
        let &Self(sid) = self;
        sid
    }
}
