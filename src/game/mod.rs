mod client;
pub mod local;
pub mod multiplayer;
mod server;
pub mod session;
mod types;

use crate::coordinations::*;
use types::*;

use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SessionUserID(u8);

impl SessionUserID {
    pub fn new(uid: u8) -> Self {
        Self(uid)
    }

    pub fn value(&self) -> u8 {
        let &Self(uid) = self;
        uid
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hint(pub u8);

#[derive(Clone, Copy)]
pub struct Mines(pub u32);

#[derive(Clone, Copy)]
pub struct Lives(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Covered,
    HintBy(Hint, SessionUserID),
    FlaggedBy(SessionUserID),
    Mine,
    ExplodedBy(SessionUserID),
    FalseFlaggedBy(SessionUserID),
    QuestionMarked(SessionUserID),
}

#[derive(Copy, Clone, Default, Debug)]
pub struct UserStat {
    pub marked_correct: u32,
    pub marked_incorrect: u32,
    pub exploded: u32,
}

#[derive(Clone, Default, Debug)]
pub struct UserStats(pub HashMap<SessionUserID, UserStat>);

#[derive(Debug, Copy, Clone)]
pub struct Stats {
    pub mines_left: i32,
    pub lives_left: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum Status {
    Playing,
    Ended { success: bool },
}

impl Status {
    pub fn done(&self) -> bool {
        match *self {
            Status::Playing => false,
            Status::Ended { .. } => true,
        }
    }
}

pub trait LocalPlayerListener {
    fn on_left_click(&mut self, coord: &Coord);
    fn on_right_click(&mut self, coord: &Coord);
}

pub trait FieldProvider {
    fn get_cell(&self, coord: &Coord) -> Cell;
}
