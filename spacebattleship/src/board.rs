//! Types that make up the game board.

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use crate::ships::{ShapeProjection, ShipId};

use self::grid::Grid;
pub use self::{
    dimensions::{ColinearCheck, Coordinate, Dimensions, NeighborIter, NeighborIterState},
    errors::{AddShipError, CannotPlaceReason, CannotShootReason, PlaceError, ShotError},
    setup::BoardSetup,
};

pub mod common;
mod dimensions;
mod errors;
mod grid;
pub mod rectangular;
pub mod setup;

/// Handle to a ship that allows getting information about its status.
#[derive(Debug)]
pub struct ShipRef<'a, I, D: Dimensions> {
    /// ID of the ship.
    id: &'a I,

    /// Grid from the board.
    grid: &'a Grid<I, D>,

    /// Projected shape of the ship.
    shape: &'a ShapeProjection<D::Coordinate>,
}

impl<'a, I: ShipId, D: Dimensions> ShipRef<'a, I, D> {
    /// Get the ID of the ship.
    pub fn id(&self) -> &'a I {
        self.id
    }

    /// Check if this ship has been sunk.
    pub fn sunk(&self) -> bool {
        self.coords().all(|coord| self.grid[coord].hit)
    }

    /// Get an iterator over the coordinates of this ship.
    pub fn coords(&self) -> impl 'a + Iterator<Item = &'a D::Coordinate> {
        self.shape.iter()
    }

    /// Get an iterator over the coordinates of this ship and whether those coords have
    /// been hit.
    pub fn hits(&self) -> impl 'a + Iterator<Item = (&'a D::Coordinate, bool)> {
        let grid = self.grid;
        self.coords().map(move |coord| (coord, grid[coord].hit))
    }
}

// Derive for Copy/Clone include bounds on the generic parameters, however, we can
// implement copy and clone regardless of whether our generics do.
impl<I, D: Dimensions> Clone for ShipRef<'_, I, D> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            grid: self.grid,
            shape: self.shape,
        }
    }
}
impl<I, D: Dimensions> Copy for ShipRef<'_, I, D> {}

/// Reference to a particular cell in the grid.
#[derive(Debug, Copy, Clone)]
pub struct CellRef<'a, I, D: Dimensions> {
    /// Coordinate of this cell.
    coord: D::Coordinate,

    /// Whether this cell was hit.
    hit: bool,

    /// Reference to the ship that occupies this cell if any.
    ship: Option<ShipRef<'a, I, D>>,
}

impl<'a, I, D: Dimensions> CellRef<'a, I, D> {
    /// The grid coordinate of this cell.
    pub fn coord(&self) -> &D::Coordinate {
        &self.coord
    }

    /// Whether this cell has been hit previously.
    pub fn hit(&self) -> bool {
        self.hit
    }

    /// The ship reference for the ship that occupies this cell, if any.
    pub fn ship(&self) -> Option<ShipRef<'a, I, D>> {
        self.ship
    }
}

/// Result of a shot on a single player's board.
pub enum ShotOutcome<I> {
    /// The shot did not hit anything.
    Miss,
    /// The shot hit the ship with the given ID, but did not sink it.
    Hit(I),
    /// The shot hit the ship with the given ID, but the player has more ships left.
    Sunk(I),
    /// The shot hit the ship with the given ID, and all of the player's ships are now
    /// sunk.
    Defeated(I),
}

impl<I> ShotOutcome<I> {
    /// Get the id of the ship that was hit.
    pub fn ship(&self) -> Option<&I> {
        match self {
            ShotOutcome::Miss => None,
            ShotOutcome::Hit(ref id)
            | ShotOutcome::Sunk(ref id)
            | ShotOutcome::Defeated(ref id) => Some(id),
        }
    }

    /// Extract the id of the ship that was hit from this result.
    pub fn into_ship(self) -> Option<I> {
        match self {
            ShotOutcome::Miss => None,
            ShotOutcome::Hit(id) | ShotOutcome::Sunk(id) | ShotOutcome::Defeated(id) => Some(id),
        }
    }
}

/// Represents a single player's board, including their ships and their side of the ocean.
pub struct Board<I: ShipId, D: Dimensions> {
    /// Grid of cells occupied by ships.
    grid: Grid<I, D>,

    // TODO: possible optimizations:
    // - track live vs sunk ships separately so we don't have to iterate all ships to
    //   decide if defeated or not.
    // - track number of hits on each ship independently of projection so we can
    //   efficiently decide if it was sunk. Requires deduplicating projected points.
    /// Mapping of all ship IDs to their projected positions in the grid.
    ships: HashMap<I, ShapeProjection<D::Coordinate>>,
}

impl<I: ShipId, D: Dimensions> Board<I, D> {
    /// Get the [`Dimesnsions`] of this [`Board`].
    pub fn dimensions(&self) -> &D {
        &self.grid.dim
    }

    /// Returns true if all of this player's ships have been sunk.
    pub fn defeated(&self) -> bool {
        self.iter_ships().all(|ship| ship.sunk())
    }

    /// Get an iterator over all ships on this board.
    pub fn iter_ships(&self) -> impl Iterator<Item = ShipRef<I, D>> {
        let grid = &self.grid;
        self.ships
            .iter()
            .map(move |(id, shape)| ShipRef { id, grid, shape })
    }

    /// Get the ship with the specified ID if it exists.
    pub fn get_ship<Q: ?Sized>(&self, ship: &Q) -> Option<ShipRef<I, D>>
    where
        I: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.ships.get_key_value(ship).map(|(id, shape)| ShipRef {
            id,
            grid: &self.grid,
            shape,
        })
    }

    /// Get a reference to the cell at the given coordinate. Returns None if the
    /// coordinate is out of bounds.
    pub fn get_coord(&self, coord: D::Coordinate) -> Option<CellRef<I, D>> {
        self.grid.get(&coord).map(|cell| CellRef {
            coord,
            hit: cell.hit,
            ship: cell.ship.as_ref().map(|id| self.get_ship(id).unwrap()),
        })
    }

    /// Fire a shot at this player, returning a result indicating why the shot was aborted
    /// or the result of the shot on this player.
    pub fn shoot(
        &mut self,
        coord: D::Coordinate,
    ) -> Result<ShotOutcome<I>, ShotError<D::Coordinate>> {
        if self.defeated() {
            return Err(ShotError::new(CannotShootReason::AlreadyDefeated, coord));
        }
        let hit_ship = match self.grid.get_mut(&coord) {
            None => return Err(ShotError::new(CannotShootReason::OutOfBounds, coord)),
            Some(cell) if cell.hit => {
                return Err(ShotError::new(CannotShootReason::AlreadyShot, coord))
            }
            Some(cell) => {
                cell.hit = true;
                cell.ship.as_ref().cloned()
            }
        };
        Ok(match hit_ship {
            None => ShotOutcome::Miss,
            Some(ship) if self.defeated() => ShotOutcome::Defeated(ship),
            Some(ship) if self.get_ship(&ship).unwrap().sunk() => ShotOutcome::Sunk(ship),
            Some(ship) => ShotOutcome::Hit(ship),
        })
    }
}
