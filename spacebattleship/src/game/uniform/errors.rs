// Copyright 2020 Zachary Stewart
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt::{self, Debug};

use thiserror::Error;

use crate::board::{CannotShootReason as BoardCannotShootReason, ShotError as BoardShotError};

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

/// Reason why a particular tile could not be shot.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CannotShootReason {
    /// The game is already over.
    AlreadyOver,

    /// The player being attacked is the player whose turn it is.
    SelfShot,

    /// The PlayerId given is not known to the board.
    UnknownPlayer,

    /// The player being attacked is already defeated.
    AlreadyDefeated,

    /// The shot was out of bounds on the grid.
    OutOfBounds,

    /// The tile specified was already shot.
    AlreadyShot,
}

impl From<BoardCannotShootReason> for CannotShootReason {
    fn from(reason: BoardCannotShootReason) -> Self {
        match reason {
            BoardCannotShootReason::AlreadyDefeated => CannotShootReason::AlreadyDefeated,
            BoardCannotShootReason::OutOfBounds => CannotShootReason::OutOfBounds,
            BoardCannotShootReason::AlreadyShot => CannotShootReason::AlreadyShot,
        }
    }
}

/// Error returned when trying to shoot a cell.
#[derive(Debug, Error)]
#[error("could not shoot player {player:?} at cell {coord:?}: {reason:?}")]
pub struct ShotError<P: Debug, C: Debug> {
    /// Reason why the cell could not be shot.
    reason: CannotShootReason,

    /// Id of the player that was attacked.
    player: P,

    /// Coordinates that were attacked.
    coord: C,
}

impl<P: Debug, C: Debug> ShotError<P, C> {
    /// Create a [`ShotError`] from a reason, player and coordinate.
    pub(super) fn new(reason: CannotShootReason, player: P, coord: C) -> Self {
        Self {
            reason,
            player,
            coord,
        }
    }

    /// Create a [`ShotError`] by adding a player ID as context to a [`BoardShotError`].
    pub(super) fn add_context(cause: BoardShotError<C>, player: P) -> Self {
        Self {
            reason: cause.reason().into(),
            player,
            coord: cause.into_coord(),
        }
    }

    /// Get the reason the shot failed.
    pub fn reason(&self) -> CannotShootReason {
        self.reason
    }

    /// Get the ID of the player that was shot at.
    pub fn player(&self) -> &P {
        &self.player
    }

    /// Get the coordinate that was shot at.
    pub fn coord(&self) -> &C {
        &self.coord
    }

    /// Extract the player ID and coordinates from the error.
    pub fn into_inner(self) -> (P, C) {
        (self.player, self.coord)
    }
}
