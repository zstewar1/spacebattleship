use crate::board::Coordinate;

/// The corrdinates of a [`GridCell`][crate::board::GridCell] in the board.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coordinate2D {
    /// Horizontal position of the cell.
    pub x: usize,
    /// Vertical position of the cell.
    pub y: usize,
}

impl Coordinate2D {
    /// Construct a [`Coordinate2D`] from the given `x` and `y`.
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Coordinate for Coordinate2D {}

impl From<(usize, usize)> for Coordinate2D {
    /// Construct a [`Coordinate2D`] from the given `(x, y)` pair.
    fn from((x, y): (usize, usize)) -> Self {
        Self::new(x, y)
    }
}

impl From<Coordinate2D> for (usize, usize) {
    /// Convert the [`Coordinate2D`] into an `(x, y)` pair.
    fn from(coord: Coordinate2D) -> Self {
        (coord.x, coord.y)
    }
}
