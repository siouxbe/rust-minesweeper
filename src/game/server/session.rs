use super::*;
use crate::game::server::field;

pub struct Session {
    field: field::Field,
    mines: Mines,
    lives: Lives,
    progress: field::Progress,
}

impl Session {
    pub fn new(coords: Coordinations, mines: Mines, lives: Lives) -> Self {
        let progress = field::Progress {
            remaining_covered: 1,
            ..Default::default()
        }; // can't let it by default be done
        let field = field::Field::new(coords, mines);
        Self {
            field,
            mines,
            lives,
            progress,
        }
    }

    pub fn uncover(&mut self, coord: &Coord, uid: SessionUserID) -> CellUpdates {
        if self.status().done() {
            return CellUpdates::default();
        }
        let mut updates = self.field.uncover(coord, uid);
        if let Some(last_updates) = self.update_progress() {
            updates = last_updates
        }
        updates
    }

    pub fn toggle_mark(&mut self, coord: &Coord, uid: SessionUserID) -> CellUpdates {
        if self.status().done() {
            return CellUpdates::default();
        }
        let update = self.field.toggle_mark(coord, uid);
        let last_updates = self.update_progress();
        if let Some(last_updates) = last_updates {
            last_updates
        } else {
            update.map(CellUpdates::one).unwrap_or_default()
        }
    }

    pub fn status(&self) -> Status {
        let p = &self.progress;
        let stats = self.stats();
        if stats.lives_left == 0 {
            Status::Ended { success: false }
        } else if p.remaining_mines == 0 && p.remaining_covered == 0 {
            Status::Ended { success: true }
        } else {
            Status::Playing
        }
    }

    pub fn user_stats(&self) -> UserStats {
        self.field.user_stats()
    }

    pub fn stats(&self) -> Stats {
        let Lives(lives) = self.lives;
        let lives_left = lives - self.progress.exploded;
        let Mines(mines) = self.mines;
        let p = &self.progress;
        let mines_left = mines as i32 - p.total_flags as i32 - p.exploded as i32;
        Stats {
            mines_left,
            lives_left,
        }
    }

    fn update_progress(&mut self) -> Option<CellUpdates> {
        self.progress = self.field.progress();
        self.status().done().then(|| {
            self.field.reveal_all();
            self.field.all()
        })
    }

    pub fn coords(&self) -> Coordinations {
        self.field.coords()
    }

    pub fn all(&self) -> CellUpdates {
        self.field.all()
    }
}
