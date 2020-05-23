//! Errors used by the `Board` and `SetupBoard`.

use std::fmt::{self, Debug};

use thiserror::Error;

/// Error returned when trying to add a ship that already existed.
#[derive(Error)]
#[error("ship with id {id:?} already exists")]
pub struct AddShipError<I: Debug, S> {
    /// ID of the ship that was attempted to be added.
    id: I,
    /// The shape that was not added because another shape with the same ID already
    /// existed.
    shape: S,
}

impl<I: Debug, S> AddShipError<I, S> {
    /// Create an [`AddShipError`] for the ship with the given ID and shape.
    pub(super) fn new(id: I, shape: S) -> Self {
        Self { id, shape }
    }

    /// The id that was added.
    pub fn id(&self) -> &I {
        &self.id
    }

    /// The shape that was added.
    pub fn shape(&self) -> &S {
        &self.shape
    }

    /// Extract the ID and Shape from this error.
    pub fn into_inner(self) -> (I, S) {
        (self.id, self.shape)
    }
}

impl<I: Debug, S> From<AddShipError<I, S>> for (I, S) {
    /// Allows retrieving the inner id and shape from the error with into.
    fn from(err: AddShipError<I, S>) -> Self {
        err.into_inner()
    }
}

impl<I: Debug, S> Debug for AddShipError<I, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Reason why a ship could not be placed with a given projection.
#[derive(Debug, Error, Copy, Clone, Eq, PartialEq)]
pub enum CannotPlaceReason {
    /// The given ShipId was already placed.
    #[error("ship was already placed")]
    AlreadyPlaced,
    /// The projection provided was not a valid projection of the specified ship.
    #[error("the projection provided was not valid")]
    InvalidProjection,
    /// One or more of the cells in the projection was already occupied.
    #[error("the requested position was already occupied")]
    AlreadyOccupied,
}

/// Error caused when attempting to place a ship in an invalid position.
#[derive(Error)]
#[error("could not place ship: {reason:?}")]
pub struct PlaceError<P> {
    #[source]
    reason: CannotPlaceReason,
    placement: P,
}

impl<P> Debug for PlaceError<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<P> PlaceError<P> {
    /// Construct a placement error from a reason, ID, and placement.
    pub(super) fn new(reason: CannotPlaceReason, placement: P) -> Self {
        Self { reason, placement }
    }

    /// Get the reason placement was aborted.
    pub fn reason(&self) -> CannotPlaceReason {
        self.reason
    }

    /// Get a reference to the [`ShapeProjection`] where placement was attempted.
    pub fn placement(&self) -> &P {
        &self.placement
    }

    /// Extract the Placement from this error.
    pub fn into_placement(self) -> P {
        self.placement
    }
}

/// Reason why a particular tile could not be shot.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CannotShootReason {
    /// The player being attacked was already defeated.
    AlreadyDefeated,

    /// The cell selected was out of bounds on the board.
    OutOfBounds,

    /// A shot has already been fired at that cell.
    AlreadyShot,
}

/// Error returned when trying to shoot a cell.
#[derive(Debug, Error)]
#[error("could not shoot cell {coord:?}: {reason:?}")]
pub struct ShotError<C: Debug> {
    /// Reason why the cell could not be shot.
    reason: CannotShootReason,

    /// The coordinates of the cell.
    coord: C,
}

impl<C: Debug> ShotError<C> {
    /// Construct a shot error with the given reason for the specified cell.
    pub(super) fn new(reason: CannotShootReason, coord: C) -> Self {
        Self { reason, coord }
    }

    /// Get the reason the shot failed.
    pub fn reason(&self) -> CannotShootReason {
        self.reason
    }

    /// Get the coordinate of the shot cell.
    pub fn coord(&self) -> &C {
        &self.coord
    }

    /// Extract the coordinate of the shot cell.
    pub fn into_coord(self) -> C {
        self.coord
    }
}
