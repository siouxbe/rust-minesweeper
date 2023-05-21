use super::*;

use crate::coordinations::*;
use crate::graphics;
use game::session::Namer;
use game::session::Session;
use game::session::SessionManager;

pub struct Grid<'a> {
    namer: &'a dyn Namer,
    local_player_listener: &'a mut dyn game::LocalPlayerListener,
    field_provider: &'a dyn game::FieldProvider,
}

impl graphics::Grid for Grid<'_> {
    fn get_cell<'a>(&'a self, coord: &Coord) -> graphics::Cell<'a> {
        let cell = self.field_provider.get_cell(coord);
        client_cell_to_graphics_cell(&cell, &|uid| self.namer.name(uid.into()))
    }

    fn left_click_cell(&mut self, coord: &Coord) {
        self.local_player_listener.on_left_click(coord)
    }

    fn right_click_cell(&mut self, coord: &Coord) {
        self.local_player_listener.on_right_click(coord)
    }
}

pub struct Main<M>
where
    M: SessionManager,
{
    status: Status<M::Session>,
    requested_status: RequestedStatus,
    manager: M,
}

impl<M> Main<M>
where
    M: SessionManager,
{
    pub fn new(manager: M) -> Self {
        let session = manager.request_new_session();
        let status = Status::Running(session);
        let requested_status = RequestedStatus::Running;
        Self {
            manager,
            requested_status,
            status,
        }
    }

    pub fn exec(self) {
        graphics::run_window(graphics::WINDOW_DEFAULT_TITLE, self)
    }
}

impl<M> graphics::StatusGenerator for Main<M>
where
    M: SessionManager,
{
    type Grid<'a> = Grid<'a>;
    type Controller<'a> = RunningStatusRequester<'a>;

    fn status<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(graphics::Status<'a, Self::Controller<'a>, Self::Grid<'a>>) -> R,
    {
        match (&self.status, &self.requested_status) {
            (Status::Done { .. }, RequestedStatus::Running) => {
                let session = self.manager.request_new_session();
                self.status = Status::Running(session);
            }
            (Status::Running(_), RequestedStatus::Done) => {
                panic!("You can't request for the game to be done!");
            }
            (Status::Running(_), RequestedStatus::Running) => {}
            (Status::Done { .. }, RequestedStatus::Done) => {}
        }
        match &mut self.status {
            Status::Running(session) => session
                .snapshot(|snapshot| -> Result<R, (F, graphics::Statistics, bool)> {
                    let game::session::SessionSnapshot {
                        stats:
                            game::Stats {
                                lives_left,
                                mines_left,
                            },
                        user_stats,
                        status,
                        coords,
                        namer,
                        field_provider,
                        local_player_listener,
                    } = snapshot;
                    if let game::Status::Ended { success } = status {
                        let stats = to_graphics_stats(user_stats, namer);
                        return Err((f, stats, success));
                    }
                    let lives_left = graphics::LivesLeft(lives_left);
                    let mines_left = graphics::MinesLeft(mines_left);
                    let grid = Grid {
                        local_player_listener,
                        field_provider,
                        namer,
                    };
                    Ok(f(graphics::Status::Active(graphics::Active {
                        coords,
                        grid,
                        lives_left,
                        mines_left,
                    })))
                })
                .unwrap_or_else(|(f, stats, success)| {
                    self.status = Status::Done { stats, success };
                    self.requested_status = RequestedStatus::Done;
                    self.status(f)
                }),
            Status::Done { stats, success } => {
                let nonactive = graphics::NonActive {
                    controller: RunningStatusRequester::new(&mut self.requested_status),
                    stats,
                };
                let success = *success;
                f(graphics::Status::NonActive { nonactive, success })
            }
        }
    }
}

pub struct RunningStatusRequester<'a>(&'a mut RequestedStatus);

impl<'a> RunningStatusRequester<'a> {
    fn new(status: &'a mut RequestedStatus) -> Self {
        *status = RequestedStatus::Done;
        Self(status)
    }
}

impl<'a> graphics::Controller for RunningStatusRequester<'a> {
    fn request_new_game(&mut self) {
        let Self(status) = self;
        **status = RequestedStatus::Running;
    }
}

enum Status<S>
where
    S: Session,
{
    Running(S),
    Done {
        stats: graphics::Statistics,
        success: bool,
    },
}

enum RequestedStatus {
    Running,
    Done,
}

fn client_cell_to_graphics_cell<'a, F>(cell: &game::Cell, names: &F) -> graphics::Cell<'a>
where
    F: Fn(graphics::PlayerID) -> &'a str,
{
    let to_player = |uid: game::SessionUserID| -> graphics::Player {
        let id = uid.into();
        let name = names(id);
        graphics::Player { id, name }
    };
    match *cell {
        game::Cell::Covered => graphics::Cell::Covered,
        game::Cell::Mine => graphics::Cell::Mine,
        game::Cell::HintBy(game::Hint(0), _uid) => graphics::Cell::EmptyNone,
        game::Cell::HintBy(game::Hint(1), _uid) => graphics::Cell::EmptyOne,
        game::Cell::HintBy(game::Hint(2), _uid) => graphics::Cell::EmptyTwo,
        game::Cell::HintBy(game::Hint(3), _uid) => graphics::Cell::EmptyThree,
        game::Cell::HintBy(game::Hint(4), _uid) => graphics::Cell::EmptyFour,
        game::Cell::HintBy(game::Hint(5), _uid) => graphics::Cell::EmptyFive,
        game::Cell::HintBy(game::Hint(6), _uid) => graphics::Cell::EmptySix,
        game::Cell::HintBy(game::Hint(7), _uid) => graphics::Cell::EmptySeven,
        game::Cell::HintBy(game::Hint(8), _uid) => graphics::Cell::EmptyEight,
        game::Cell::HintBy(game::Hint(_), _uid) => {
            panic!("A cell on a square grid can not be surrounded by 9 mines or more")
        }
        game::Cell::QuestionMarked(uid) => graphics::Cell::Maybe(to_player(uid)),
        game::Cell::FlaggedBy(uid) => graphics::Cell::Flag(to_player(uid)),
        game::Cell::ExplodedBy(uid) => graphics::Cell::ExplodedMine(to_player(uid)),
        game::Cell::FalseFlaggedBy(uid) => graphics::Cell::Incorrect(to_player(uid)),
    }
}

fn to_graphics_stat<S>(
    stats: &game::UserStats,
    namer: &dyn Namer,
    number: S,
) -> Vec<graphics::UserStat>
where
    S: Fn(&game::UserStat) -> u32,
{
    let game::UserStats(stats) = stats;
    let mut stats: Vec<graphics::UserStat> = stats
        .iter()
        .map(|(&uid, stat)| graphics::UserStat {
            id: uid.into(),
            name: namer.name(uid).into(),
            number: number(stat),
        })
        .collect();
    stats.sort_by(|a, b| b.number.cmp(&a.number));
    stats
}

fn to_graphics_stats(stats: game::UserStats, namer: &dyn Namer) -> graphics::Statistics {
    let marked_correct = to_graphics_stat(&stats, namer, |stat| stat.marked_correct);
    let marked_incorrect = to_graphics_stat(&stats, namer, |stat| stat.marked_incorrect);
    let exploded = to_graphics_stat(&stats, namer, |stat| stat.exploded);
    graphics::Statistics {
        exploded,
        marked_correct,
        marked_incorrect,
    }
}

impl From<graphics::PlayerID> for game::SessionUserID {
    fn from(id: graphics::PlayerID) -> Self {
        let graphics::PlayerID(id) = id;
        Self::new(id)
    }
}

impl From<game::SessionUserID> for graphics::PlayerID {
    fn from(id: game::SessionUserID) -> Self {
        Self(id.value())
    }
}
