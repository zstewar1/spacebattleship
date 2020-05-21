//! Errors used by the `Board` and `SetupBoard`.

use std::fmt;

use thiserror::Error;

use crate::ships::ShapeProjection;

/// Reason why a ship could not be placed with a given projection.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CannotPlaceReason {
    /// The given ShipId was already placed.
    AlreadyPlaced,
    /// The projection provided was not a valid projection of the specified ship.
    InvalidProjection,
    /// One or more of the cells in the projection was already occupied.
    AlreadyOccupied,
}

/// Error caused when attempting to place a ship in an invalid position.
#[derive(Debug, Error)]
#[error("could not place ship {id:?}: {reason:?}")]
pub struct PlaceError<I: fmt::Debug, C: fmt::Debug> {
    reason: CannotPlaceReason,
    id: I,
    placement: ShapeProjection<C>,
}

impl<I: fmt::Debug, C: fmt::Debug> PlaceError<I, C> {
    /// Construct a placement error from a reason, ID, and placement.
    pub(super) fn new(reason: CannotPlaceReason, id: I, placement: ShapeProjection<C>) -> Self {
        Self {
            reason,
            id,
            placement,
        }
    }

    /// Get the reason placement was aborted.
    pub fn reason(&self) -> CannotPlaceReason {
        self.reason
    }

    /// Get a reference to the [`ShapeProjection`] where placement was attempted.
    pub fn placement(&self) -> &ShapeProjection<C> {
        &self.placement
    }

    /// Extract the Placement from this error.
    pub fn into_placement(self) -> ShapeProjection<C> {
        self.placement
    }
}
