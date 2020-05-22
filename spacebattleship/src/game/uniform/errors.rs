use std::fmt::{self, Debug};

use thiserror::Error;

/// Error returned when trying to add a ship that already existed.
#[derive(Error)]
#[error("player with id {id:?} already exists")]
pub struct AddPlayerError<P: Debug, D> {
    /// ID of the player that was attempted to be added.
    id: P,
    /// The dimensions of the player grid that was not added because the player ID was
    /// already in use.
    dim: D,
}

impl<P: Debug, D> AddPlayerError<P, D> {
    /// Create an [`AddPlayerError`] for the player with the given ID and dimensions.
    pub(super) fn new(id: P, dim: D) -> Self {
        Self { id, dim }
    }

    /// The id of the player that was added.
    pub fn id(&self) -> &P {
        &self.id
    }

    /// The dimensions of the board that was attempted to be added.
    pub fn dimensions(&self) -> &D {
        &self.dim
    }

    /// Extract the ID and Dimensions from this error.
    pub fn into_inner(self) -> (P, D) {
        (self.id, self.dim)
    }
}

impl<P: Debug, D> From<AddPlayerError<P, D>> for (P, D) {
    /// Allows retrieving the inner id and shape from the error with into.
    fn from(err: AddPlayerError<P, D>) -> Self {
        err.into_inner()
    }
}

impl<I: Debug, S> Debug for AddPlayerError<I, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
