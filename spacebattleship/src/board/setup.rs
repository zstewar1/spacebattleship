//! Implements the setup phase of the board.
use std::{
    collections::{hash_map::Entry, HashMap},
};

use crate::{
    board::{AddShipError, Board, CannotPlaceReason, Dimensions, Grid, PlaceError},
    ships::{ProjectIter, ShapeProjection, ShipId, ShipShape},
};

/// Reference to a particular ship's placement info as well as the grid, providing access
/// to the methods necessary to check it's placement status and place or unplace it.
pub struct ShipEntry<'a, I, D: Dimensions, S> {
    /// ID of this
    id: I,

    /// Grid that ships are being placed into.
    grid: &'a mut Grid<I, D>,

    /// Back ref to the ship.
    ship: &'a mut ShipPlacementInfo<S, D::Coordinate>,
}

impl<'a, I: ShipId, D: Dimensions, S: ShipShape<D>> ShipEntry<'a, I, D, S> {
    /// Returns true if this ship has been placed.
    pub fn placed(&self) -> bool {
        self.ship.placement.is_some()
    }

    /// If the ship is placed, get the placement. Otherwise return `None`.
    pub fn placement(&self) -> Option<&ShapeProjection<D::Coordinate>> {
        self.ship.placement.as_ref()
    }

    /// Get an interator over possible projections of the shape for this ship that
    /// start from the given [`Coordinate`]. If there are no possible placements
    /// from the given coordinate, including if the coordinate is out of bounds,
    /// the resulting iterator will be empty.
    pub fn get_placements(&self, coord: D::Coordinate) -> ProjectIter<D, S::ProjectIterState> {
        self.ship.shape.project(coord, &self.grid.dim)
    }

    /// Attempts to place the ship with onto the given coordinates. If the ship is already
    /// placed, returns `Err` with the attempted placement and reason placement failed,
    /// otherwise returns `Ok(())`
    pub fn place(
        &mut self,
        placement: ShapeProjection<D::Coordinate>,
    ) -> Result<(), PlaceError<ShapeProjection<D::Coordinate>>> {
        if self.placed() {
            Err(PlaceError::new(CannotPlaceReason::AlreadyPlaced, placement))
        } else if !self
            .ship
            .shape
            .is_valid_placement(&placement, &self.grid.dim)
        {
            Err(PlaceError::new(
                CannotPlaceReason::InvalidProjection,
                placement,
            ))
        } else {
            for coord in placement.iter() {
                match self.grid.get(coord) {
                    None => {
                        // ShipShape should ensure that all coordinates are valid, but don't
                        // trust it.
                        return Err(PlaceError::new(
                            CannotPlaceReason::InvalidProjection,
                            placement,
                        ));
                    }
                    Some(cell) if cell.ship.is_some() => {
                        return Err(PlaceError::new(
                            CannotPlaceReason::AlreadyOccupied,
                            placement,
                        ));
                    }
                    _ => {}
                }
            }
            // Already ensured that every position is valid and not occupied.
            for coord in placement.iter() {
                self.grid[coord].ship = Some(self.id.to_owned());
            }
            self.ship.placement = Some(placement);
            Ok(())
        }
    }

    /// Attempt to clear the placement of the ship. Returns the previous placement of the
    /// ship if any. Returns `None` if the ship has not been placed.
    pub fn unplace(&mut self) -> Option<ShapeProjection<D::Coordinate>> {
        self.ship.placement.take().map(|placement| {
            for coord in placement.iter() {
                // We should only allow placement on valid cells, so unwrap is fine.
                self.grid[coord].ship = None;
            }
            placement
        })
    }
}

/// Contains a ship's shape and current placement status in the grid.
struct ShipPlacementInfo<S, C> {
    /// Shape being placed.
    shape: S,

    /// Placement of this ship, if it has been placed.
    placement: Option<ShapeProjection<C>>,
}

/// Setup phase for a [`Board`]. Allows placing ships and does not allow shooting.
pub struct BoardSetup<I: ShipId, D: Dimensions, S: ShipShape<D>> {
    /// Grid for placement of ships.
    grid: Grid<I, D>,

    /// Mapping of added ShipIds to coresponding placement info.
    ships: HashMap<I, ShipPlacementInfo<S, D::Coordinate>>,
}

impl<I: ShipId, D: Dimensions, S: ShipShape<D>> BoardSetup<I, D, S> {
    /// Begin game setup by constructing a new board with the given [`Dimensions`].
    pub fn new(dim: D) -> Self {
        Self {
            grid: Grid::new(dim),
            ships: HashMap::new(),
        }
    }

    /// Get the [`Dimesnsions`] of this [`Board`].
    pub fn dimensions(&self) -> &D {
        &self.grid.dim
    }

    /// Tries to start the game. If all ships are placed, returns a [`Board`] with the
    /// current placements. If no ships have been added or any ship has not been placed,
    /// returns self.
    pub fn start(self) -> Result<Board<I, D>, Self> {
        if !self.ready() {
            Err(self)
        } else {
            Ok(Board {
                grid: self.grid,
                ships: self
                    .ships
                    .into_iter()
                    .map(|(id, mut info)| (id, info.placement.take().unwrap()))
                    .collect(),
            })
        }
    }

    /// Checks if this board is ready to start. Returns `true` if at least one ship has
    /// been added and all ships are placed.
    pub fn ready(&self) -> bool {
        !self.ships.is_empty() && self.ships.values().all(|ship| ship.placement.is_some())
    }

    /// Get an iterator over the IDs of any ships which still need to be placed.
    pub fn pending_ships(&self) -> impl Iterator<Item = &I> {
        self.ships.iter().filter_map(
            |(id, ship)| {
                if ship.placement.is_some() {
                    None
                } else {
                    Some(id)
                }
            },
        )
    }

    /// Attempts to add a ship with the given ID. If the given ShipID is already used,
    /// returns the shape passed to this function. Otherwise adds the shape and returns
    /// the ShipEntry for it to allow placement.
    pub fn add_ship(&mut self, id: I, shape: S) -> Result<ShipEntry<I, D, S>, AddShipError<I, S>> {
        match self.ships.entry(id.clone()) {
            Entry::Occupied(_) => Err(AddShipError::new(id, shape)),
            Entry::Vacant(entry) => {
                let ship = entry.insert(ShipPlacementInfo {
                    shape,
                    placement: None,
                });
                Ok(ShipEntry {
                    id,
                    grid: &mut self.grid,
                    ship,
                })
            }
        }
    }

    /// Get the [`ShipEntry`] for the ship with the specified ID if such a ship exists.
    pub fn get_ship(&mut self, id: I) -> Option<ShipEntry<I, D, S>> {
        let grid = &mut self.grid;
        self.ships
            .get_mut(&id)
            .map(move |ship| ShipEntry { id, grid, ship })
    }
}
