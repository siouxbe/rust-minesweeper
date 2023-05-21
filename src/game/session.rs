use super::*;

#[derive(Copy, Clone)]
pub struct SessionConfig {
    pub coords: Coordinations,
    pub mines: Mines,
    pub lives: Lives,
}

pub struct SessionSnapshot<'a> {
    pub stats: Stats,
    pub user_stats: UserStats,
    pub status: Status,
    pub coords: Coordinations,
    pub namer: &'a dyn Namer,
    pub field_provider: &'a dyn FieldProvider,
    pub local_player_listener: &'a mut dyn LocalPlayerListener,
}

pub trait Session {
    fn snapshot<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(SessionSnapshot<'a>) -> R;
}

pub trait SessionManager {
    type Session: Session;

    fn request_new_session(&self) -> Self::Session;
}

pub trait Namer {
    fn name(&self, uid: SessionUserID) -> &str;
}
