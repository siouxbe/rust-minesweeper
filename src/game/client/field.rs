use super::*;

pub struct Field {
    coords: Coordinations,
    cells: Vec<Cell>,
}

impl Field {
    pub fn new(coords: Coordinations) -> Self {
        let mut cells = Vec::new();
        cells.resize(coords.size(), Cell::Covered);
        Self { coords, cells }
    }

    pub fn get_cell(&self, coord: &Coord) -> &Cell {
        let Index(index) = self.coords.to_index(coord).expect("Invalid coordinates");
        &self.cells[index]
    }

    fn on_update(&mut self, update: CellUpdate) {
        let CellUpdate { cell, coord } = update;
        let Index(index) = self.coords.to_index(&coord).expect("Invalid coordinates");
        self.cells[index] = cell;
    }

    pub fn on_updates(&mut self, updates: CellUpdates) {
        let CellUpdates(updates) = updates;
        for update in updates {
            self.on_update(update)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COORDS: Coordinations = Coordinations::from_width_and_height(10, 8);
    const COORD: Coord = Coord { x: 3, y: 6 };
    const INDEX: usize = 63;
    const COORD2: Coord = Coord { x: 4, y: 7 };
    const INDEX2: usize = 74;
    const COORD_INVALID: Coord = Coord { x: 99, y: 6 };

    #[test]
    fn number_of_cells_equals_width_times_height() {
        let field = Field::new(COORDS);
        assert_eq!(field.cells.len(), 80);
    }

    #[test]
    fn initally_all_cells_are_covered() {
        let field = Field::new(COORDS);
        assert!(field
            .cells
            .iter()
            .all(|cell| matches!(*cell, Cell::Covered)));
    }

    fn updated_field(coord: Coord) -> Field {
        let mut field = Field::new(COORDS);
        let update = CellUpdate {
            coord,
            cell: Cell::Mine,
        };
        field.on_update(update);
        field
    }

    fn updated_field2(a: Coord, b: Coord) -> Field {
        let mut field = Field::new(COORDS);
        let a = CellUpdate {
            coord: a,
            cell: Cell::Mine,
        };
        let b = CellUpdate {
            coord: b,
            cell: Cell::Mine,
        };
        field.on_updates(CellUpdates(vec![a, b]));
        field
    }

    fn compare_field_cells(field: &Field, cells: &Vec<Cell>) {
        assert_eq!(field.cells.len(), cells.len());
        assert!(field.cells.iter().zip(cells.iter()).all(|(a, b)| *a == *b));
    }

    #[test]
    fn on_update_updates_the_correct_cell() {
        let field = updated_field(COORD);
        let cells = {
            let mut cells = vec![Cell::Covered; 80];
            cells[INDEX] = Cell::Mine;
            cells
        };
        compare_field_cells(&field, &cells);
    }

    #[test]
    #[should_panic]
    fn on_update_panics_when_invalid_coord() {
        updated_field(COORD_INVALID);
    }

    #[test]
    fn get_cell_returns_the_correct_cell() {
        let field = updated_field(COORD);
        let mine = field.get_cell(&COORD);
        assert_eq!(*mine, Cell::Mine);
    }

    #[test]
    #[should_panic]
    fn get_cell_panics_when_invalid_coord() {
        let field = updated_field(COORD);
        field.get_cell(&COORD_INVALID);
    }

    #[test]
    fn on_updates_updates_the_correct_cells() {
        let field = updated_field2(COORD, COORD2);
        let cells = {
            let mut cells = vec![Cell::Covered; 80];
            for index in [INDEX, INDEX2] {
                cells[index] = Cell::Mine;
            }
            cells
        };
        compare_field_cells(&field, &cells);
    }

    #[test]
    #[should_panic]
    fn on_updates_panics_when_invalid_coord() {
        updated_field2(COORD, COORD_INVALID);
    }
}
