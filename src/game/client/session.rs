use super::*;

pub struct Session {
    field: field::Field,
    stats: Stats,
}

impl Session {
    pub fn new(coords: Coordinations, mines: Mines, lives: Lives) -> Self {
        let stats = {
            let Mines(mines_left) = mines;
            let Lives(lives_left) = lives;
            Stats {
                mines_left: mines_left as i32,
                lives_left,
            }
        };
        let field = field::Field::new(coords);
        Self { field, stats }
    }

    pub fn blank(coords: Coordinations, stats: Stats) -> Self {
        let field = field::Field::new(coords);
        Self { field, stats }
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    fn on_stat_update(&mut self, stats: Stats) {
        self.stats = stats;
    }
}

impl FieldProvider for Session {
    fn get_cell(&self, coord: &Coord) -> Cell {
        *self.field.get_cell(coord)
    }
}

impl UpdatesListener for Session {
    fn on_updates(&mut self, updates: Updates) {
        let Updates { cells, stats } = updates;
        self.on_stat_update(stats);
        self.field.on_updates(cells)
    }
}
