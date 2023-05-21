//! This crate allows for easily identifying cells within a mines field.
//! Cells are identified using either an index or a coordinate.
//! Currently the only kind of field that is supported is a rectangular field.

/// Any given coordinate can not have more neighboring coordinates than this number.
pub const MAX_SURROUNDING_MINES: u8 = 8;

/// Represents a rectangular grid.
///
/// X coordinations navigate from left to right, Y coordinations navigate from top to bottom.
#[derive(Clone, Copy, Debug)]
pub struct Coordinations {
    width: u32,
    height: u32,
}

impl Coordinations {
    pub const fn from_width_and_height(width: u32, height: u32) -> Self {
        todo!()
    }

    pub fn rows(&self) -> u32 {
        todo!()
    }

    pub fn columns(&self) -> u32 {
        todo!()
    }

    pub fn w_over_h(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Returns the total number of distinct coordinates within this field.
    ///```
    /// # use sioux_rust_minesweeper_crate::coordinations::*;
    ///let c = Coordinations::from_width_and_height(5, 3);
    ///assert_eq!(c.size(), 15);
    ///```
    pub fn size(&self) -> usize {
        todo!()
    }

    /// Converts an index to a coordinate.
    /// Returns `None` if the index is invalid.
    ///```
    /// # use sioux_rust_minesweeper_crate::coordinations::*;
    ///let c = Coordinations::from_width_and_height(5, 3);
    ///assert_eq!(c.to_coord(Index(9)), Some(Coord{x: 4, y: 1}));
    ///```
    pub fn to_coord(&self, index: Index) -> Option<Coord> {
        todo!()
    }

    /// Converts a coordinate to an index.
    /// Returns `None` if the coordinate is invalid.
    ///```
    /// # use sioux_rust_minesweeper_crate::coordinations::*;
    ///let c = Coordinations::from_width_and_height(5, 3);
    ///assert_eq!(c.to_index(&Coord{x: 4, y: 1}), Some(Index(9)));
    ///```
    pub fn to_index(&self, coord: &Coord) -> Option<Index> {
        todo!()
    }

    /// Returns an iterator over all neighboring indices.
    #[allow(unused_variables)]
    pub fn neighbors_at_index(&self, index: Index) -> impl Iterator<Item = Index> {
        std::iter::empty()
    }

    /// Determines whether or not a coordinate can point to an existing element within the width
    /// and height.
    fn inside(&self, coord: &Coord) -> bool {
        coord.x < self.width && coord.y < self.height
    }
}

/// Represents a location in a rectangular field using an x coordinate and an y coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Coord {
    ///starts at the left
    pub x: u32,
    ///starts at the top
    pub y: u32,
}

/// Represents a location in a rectangular field using an index.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Index(pub usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_from_width_and_height() {
        let coords = Coordinations::from_width_and_height(5, 4);
        assert_eq!(coords.width, 5);
        assert_eq!(coords.height, 4);
    }

    #[test]
    fn columns_equals_width() {
        let coords = Coordinations::from_width_and_height(5, 4);
        assert_eq!(coords.columns(), 5);
    }

    #[test]
    fn rows_equals_height() {
        let coords = Coordinations::from_width_and_height(5, 4);
        assert_eq!(coords.rows(), 4);
    }

    fn correct_neighbors(middle: Index, neighbors: &[Index]) {
        let coords = Coordinations::from_width_and_height(5, 4);
        // coords
        // 00 10 20 30 40
        // 01 11 21 31 41
        // 02 12 22 32 42
        // 03 13 23 33 43
        // indices
        // 00 01 02 03 04
        // 05 06 07 08 09
        // 10 11 12 13 14
        // 15 16 17 18 19
        let mut n = coords.neighbors_at_index(middle);
        for index in neighbors {
            assert_eq!(n.next(), Some(*index));
        }
        assert_eq!(n.next(), None);
    }

    #[test]
    fn correct_neighbors_center() {
        correct_neighbors(
            Index(7),
            &[
                Index(1),
                Index(2),
                Index(3),
                Index(6),
                Index(8),
                Index(11),
                Index(12),
                Index(13),
            ],
        );
    }

    #[test]
    fn correct_neighbors_left() {
        correct_neighbors(
            Index(5),
            &[Index(0), Index(1), Index(6), Index(10), Index(11)],
        );
    }

    #[test]
    fn correct_neighbors_right() {
        correct_neighbors(
            Index(14),
            &[Index(8), Index(9), Index(13), Index(18), Index(19)],
        );
    }

    #[test]
    fn correct_neighbors_top() {
        correct_neighbors(
            Index(3),
            &[Index(2), Index(4), Index(7), Index(8), Index(9)],
        );
    }

    #[test]
    fn correct_neighbors_bottom() {
        correct_neighbors(
            Index(18),
            &[Index(12), Index(13), Index(14), Index(17), Index(19)],
        );
    }

    #[test]
    fn correct_neighbors_top_left() {
        correct_neighbors(Index(0), &[Index(1), Index(5), Index(6)]);
    }

    #[test]
    fn correct_neighbors_top_right() {
        correct_neighbors(Index(4), &[Index(3), Index(8), Index(9)]);
    }

    #[test]
    fn correct_neighbors_bottom_left() {
        correct_neighbors(Index(15), &[Index(10), Index(11), Index(16)]);
    }

    #[test]
    fn correct_neighbors_bottom_right() {
        correct_neighbors(Index(19), &[Index(13), Index(14), Index(18)]);
    }
}
