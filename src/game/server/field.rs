use super::*;
use crate::game;

use std::collections::{HashMap, HashSet};

use rand::seq::SliceRandom;
use rand::thread_rng;

const RANDOM_FIELD: bool = true;

#[allow(clippy::upper_case_acronyms)]
type UID = SessionUserID;

#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Content {
    Mine,
    Hint(Hint),
}

impl Content {
    fn is_empty_cell(&self) -> bool {
        matches!(self, Self::Hint(Hint(0)))
    }

    fn is_actual_hint(&self) -> bool {
        match *self {
            Self::Mine => false,
            Self::Hint(Hint(0)) => false,
            Self::Hint(_) => true,
        }
    }
}

#[derive(Default, Debug)]
pub struct Progress {
    pub remaining_covered: u32,
    pub remaining_mines: u32,
    pub exploded: u32,
    pub total_flags: u32,
}

pub struct Field {
    coords: Coordinations,
    cells: Vec<Cell>,
}

impl Field {
    pub fn new(coords: Coordinations, mines: Mines) -> Self {
        let cells = Concept::random(coords, mines).to_cells();
        Self::new_with_cells(coords, cells)
    }

    fn new_with_cells(coords: Coordinations, cells: Vec<Cell>) -> Self {
        Self { coords, cells }
    }

    pub fn coords(&self) -> Coordinations {
        self.coords
    }

    pub fn progress(&self) -> Progress {
        let mut progress = Progress {
            remaining_covered: self.count(|cell| matches!(cell.status, Status::Covered)),
            remaining_mines: self.count(|cell| {
                matches!(cell.content, Content::Mine) && !matches!(cell.status, Status::MarkedBy(_))
            }),
            exploded: self.count(|cell| {
                matches!(cell.content, Content::Mine)
                    && matches!(cell.status, Status::UncoveredBy(_))
            }),
            total_flags: self.count(|cell| matches!(cell.status, Status::MarkedBy(_))),
        };
        progress.remaining_mines -= progress.exploded;
        progress
    }

    fn count<F>(&self, filter: F) -> u32
    where
        F: Fn(&Cell) -> bool,
    {
        self.cells.iter().filter(|cell| filter(cell)).count() as u32
    }

    pub fn user_stats(&self) -> UserStats {
        let mut stats: HashMap<UID, UserStat> = self
            .cells
            .iter()
            .filter_map(|cell| match cell.status {
                Status::Covered => None,
                Status::UncoveredBy(uid) => Some(uid),
                Status::MarkedBy(uid) => Some(uid),
                Status::QuestionMarkedBy(uid) => Some(uid),
                Status::EndGameCovered => None,
                Status::EndGameMarkedBy(uid) => Some(uid),
            })
            .map(|uid| (uid, UserStat::default()))
            .collect();
        for (&suid, stat) in &mut stats {
            *stat = self.cells.iter().fold(
                UserStat::default(),
                |mut stat: UserStat, cell: &Cell| -> UserStat {
                    match (cell.content, cell.status) {
                        (Content::Mine, Status::MarkedBy(uid)) if uid == suid => {
                            stat.marked_correct += 1
                        }
                        (Content::Mine, Status::EndGameMarkedBy(uid)) if uid == suid => {
                            stat.marked_correct += 1
                        }
                        (Content::Hint(_), Status::MarkedBy(uid)) if uid == suid => {
                            stat.marked_incorrect += 1
                        }
                        (Content::Hint(_), Status::EndGameMarkedBy(uid)) if uid == suid => {
                            stat.marked_incorrect += 1
                        }
                        (Content::Mine, Status::UncoveredBy(uid)) if uid == suid => {
                            stat.exploded += 1
                        }
                        _ => {}
                    }
                    stat
                },
            );
        }
        UserStats(stats)
    }

    pub fn uncover(&mut self, coord: &Coord, uid: UID) -> CellUpdates {
        let ci = match self.coords.to_index(coord) {
            Some(index) => index,
            None => return CellUpdates::none(),
        };
        let Index(index) = ci;
        let cell = &mut self.cells[index];
        if let Status::UncoveredBy(_) = &cell.status {
            return CellUpdates::none();
        }
        cell.status = Status::UncoveredBy(uid);
        if let Content::Mine = &cell.content {}
        if !cell.content.is_empty_cell() {
            let cell = (*cell).into();
            let update = CellUpdate {
                cell,
                coord: *coord,
            };
            return CellUpdates(vec![update]);
        }
        let updated_cells = self
            .gather_all_updated_cell_indices(ci)
            .into_iter()
            .map(|i| {
                let Index(index) = i;
                let cell = &mut self.cells[index];
                cell.status = Status::UncoveredBy(uid);
                let cell = (*cell).into();
                let coord = self.coords.to_coord(i).expect("invalid index");
                CellUpdate { cell, coord }
            })
            .collect();
        CellUpdates(updated_cells)
    }

    fn gather_all_updated_cell_indices(&self, ci: Index) -> HashSet<Index> {
        let mut updated_cell_indices = HashSet::<Index>::new();
        let mut empty_cell_indices = HashSet::<Index>::new();
        empty_cell_indices.insert(ci);
        let mut neighbors_of_empty_cell_indices = HashSet::<Index>::new();
        while !empty_cell_indices.is_empty() {
            for new_index in &empty_cell_indices {
                for neighbor_index in self.coords.neighbors_at_index(*new_index) {
                    let Index(neighbor_ci) = neighbor_index;
                    let neighbor = &self.cells[neighbor_ci];
                    if matches!(neighbor.status, Status::Covered) {
                        neighbors_of_empty_cell_indices.insert(neighbor_index);
                    } else if matches!(neighbor.content, Content::Hint(_)) {
                        updated_cell_indices.insert(neighbor_index);
                    }
                }
            }
            updated_cell_indices.extend(empty_cell_indices.drain());
            std::mem::swap(
                &mut empty_cell_indices,
                &mut neighbors_of_empty_cell_indices,
            );
            updated_cell_indices.extend(empty_cell_indices.iter().cloned().filter(|i| {
                let Index(index) = *i;
                self.cells[index].content.is_actual_hint()
            }));
            empty_cell_indices.retain(|i| {
                let Index(index) = *i;
                self.cells[index].content.is_empty_cell() && !updated_cell_indices.contains(i)
            });
        }
        updated_cell_indices
    }

    pub fn toggle_mark(&mut self, coord: &Coord, uid: UID) -> Option<CellUpdate> {
        let Index(index) = match self.coords.to_index(coord) {
            Some(index) => index,
            None => return None,
        };
        let cell = &mut self.cells[index];
        match cell.status {
            Status::UncoveredBy(_) => return None,
            Status::Covered => {
                cell.status = Status::MarkedBy(uid);
            }
            Status::MarkedBy(_) => {
                cell.status = Status::QuestionMarkedBy(uid);
            }
            Status::QuestionMarkedBy(_) => {
                cell.status = Status::Covered;
            }
            Status::EndGameCovered | Status::EndGameMarkedBy(_) => {
                panic!("Should not be able to toggle when game is played")
            }
        };
        Some(CellUpdate {
            cell: (*cell).into(),
            coord: *coord,
        })
    }

    pub fn reveal_all(&mut self) {
        for cell in &mut self.cells {
            cell.status = match cell.status {
                Status::Covered => Status::EndGameCovered,
                Status::MarkedBy(uid) => Status::EndGameMarkedBy(uid),
                _ => cell.status,
            }
        }
    }

    pub fn all(&self) -> CellUpdates {
        CellUpdates(
            self.cells
                .iter()
                .enumerate()
                .map(|(index, cell)| {
                    let coord = self
                        .coords
                        .to_coord(Index(index))
                        .expect("All cells should have correct indices");
                    CellUpdate {
                        cell: (*cell).into(),
                        coord,
                    }
                })
                .collect(),
        )
    }
}

#[derive(Clone, Copy)]
enum Status {
    Covered,
    MarkedBy(UID),
    QuestionMarkedBy(UID),
    UncoveredBy(UID),
    EndGameCovered,
    EndGameMarkedBy(UID),
}

#[derive(Clone, Copy)]
struct Cell {
    content: Content,
    status: Status,
}

impl From<game::server::field::Cell> for game::Cell {
    fn from(cell: Cell) -> Self {
        /*
         * TODO: Map the type 'sioux_rust_minesweeper_crate::game::server::field::Cell'
         * to the type 'sioux_rust_minesweeper_crate::game::Cell'.
         * You can use an if-else structure or the 'match' keyword.
         *
         * At the very least you should be able to handle the single case
         * where the content was a mine and the status was uncovered;
         * obviously this has to be mapped to the value 'ExplodedBy'.
         *
         * An other case is when the status was covered. Regardless of its content,
         * it should be mapped to covered.
         *
         * Another example: when the content was a hint and the status is
         * 'at the game end, it is still covered',
         * then it must be mapped to a hint provided by the 'SYSTEM_UID' user.
         *
         * One more example: when the content was a hint and the status is
         * 'at the game end, it was flagged',
         * then it must be mapped to 'FalseFlaggedBy'.
         *
         */
        todo!()
    }
}

#[allow(dead_code)]
struct Concept {
    mines: Vec<bool>,
    coords: Coordinations,
}

impl Concept {
    fn random(coords: Coordinations, mines: Mines) -> Self {
        let maximum_surrounding_mines = MAX_SURROUNDING_MINES as usize;
        let Mines(number_of_mines) = mines;
        let size = coords.size();
        let mut mines = Vec::<bool>::new();
        let mut indices = Vec::<Index>::new();
        let mut rng = thread_rng();
        'root: loop {
            mines.clear();
            mines.resize(size, false);

            indices.clear();
            indices.extend((0..size).map(Index));
            if RANDOM_FIELD {
                indices.shuffle(&mut rng);
            }

            for _ in 0..number_of_mines {
                let Index(index) = indices.pop().unwrap();
                mines[index] = true;
            }
            if !mines
                .iter()
                .enumerate()
                .filter(|&(_index, mine)| *mine)
                .map(|(index, _mine)| Index(index))
                .all(|index| {
                    let neighboring_mines = coords
                        .neighbors_at_index(index)
                        .filter(|Index(neighbor_index)| mines[*neighbor_index])
                        .count();
                    neighboring_mines <= maximum_surrounding_mines
                })
            {
                continue 'root;
            }
            break 'root;
        }
        Self { mines, coords }
    }

    fn to_cells(&self) -> Vec<Cell> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_concept() -> Concept {
        let coords = Coordinations::from_width_and_height(3, 3);
        let mines = vec![
            false, true, false, // first row
            true, true, false, // second row
            false, false, false, // third row
        ];
        Concept { mines, coords }
    }

    #[test]
    fn concept_to_cells() {
        let cells: Vec<Cell> = create_concept().to_cells();
        assert_eq!(cells.len(), 9);
        assert!(cells
            .iter()
            .all(|Cell { content: _, status }| matches!(status, Status::Covered)));
        let content = [
            Content::Hint(game::Hint(3)),
            Content::Mine,
            Content::Hint(game::Hint(2)), // first row
            Content::Mine,
            Content::Mine,
            Content::Hint(game::Hint(2)), // second row
            Content::Hint(game::Hint(2)),
            Content::Hint(game::Hint(2)),
            Content::Hint(game::Hint(1)), // third row
        ];
        let cell_content = cells.iter().map(|Cell { content, status: _ }| content);
        assert!(cell_content.zip(content.iter()).all(|(a, b)| a == b));
    }
}
