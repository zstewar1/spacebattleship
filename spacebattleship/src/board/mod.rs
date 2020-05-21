//! Types that make up the game board.

use std::collections::HashMap;

use crate::ships::{ShapeProjection, ShipId};

use self::grid::Grid;
pub use self::{
    dimensions::{Coordinate, Dimensions, NeighborIter, NeighborIterState},
    errors::{CannotPlaceReason, PlaceError},
    setup::BoardSetup,
};

pub mod common;
mod dimensions;
mod errors;
mod grid;
pub mod rectangular;
pub mod setup;

/// Represents a single player's board, including their ships and their side of the ocean.
pub struct Board<I: ShipId, D: Dimensions> {
    /// Grid of cells occupied by ships.
    grid: Grid<I, D>,

    /// Mapping of all ship IDs to their projected positions in the grid.
    ships: HashMap<I, ShapeProjection<D::Coordinate>>,
}

impl<I: ShipId, D: Dimensions> Board<I, D> {
    /// Get the [`Dimesnsions`] of this [`Board`].
    pub fn dimensions(&self) -> &D {
        &self.grid.dim
    }
}
