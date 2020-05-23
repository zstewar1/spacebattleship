//! Implementation of Battleship with uniform generic parameters, if not uniform board
//! setups.
use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::Hash,
};

use crate::{
    board::{Board, BoardSetup, Dimensions, ShotOutcome as BoardShotOutcome},
    ships::{ShipId, ShipShape},
};

pub use self::errors::{AddPlayerError, CannotShootReason, ShotError};

mod errors;

/// Types used for the ID of a player. IDs are treated as disposable and cheaply
/// cloneable. If you need a complex ID type that isn't cheap to clone, you may want to
/// wrap it in `Rc` or `Arc`.
///
/// Auto-implemented for any type which implements `Debug`,`Clone`, `Eq`, and `Hash`.
pub trait PlayerId: Debug + Clone + Eq + Hash {}
impl<T: Debug + Clone + Eq + Hash> PlayerId for T {}

/// Handles setup for the game. Acts as a builder for [`Game`].
pub struct GameSetup<P: PlayerId, I: ShipId, D: Dimensions, S: ShipShape<D>> {
    /// Setup boards indexed by player.
    boards: HashMap<P, BoardSetup<I, D, S>>,

    /// Records the turn order for players.
    turn_order: Vec<P>,
}

impl<P: PlayerId, I: ShipId, D: Dimensions, S: ShipShape<D>> GameSetup<P, I, D, S> {
    /// Construct a new [`GameSetup`] to build a game.
    pub fn new() -> Self {
        Self {
            boards: HashMap::new(),
            turn_order: Vec::new(),
        }
    }

    /// Tries to start the game. If all players are ready, returns a [`Game`] with the
    /// current setup. If fewer than 2 players have been added, or any player has not
    /// placed all of their ships, returns `self`.
    pub fn start(self) -> Result<Game<P, I, D>, Self> {
        if !self.ready() {
            Err(self)
        } else {
            Ok(Game {
                boards: self
                    .boards
                    .into_iter()
                    .map(|(pid, board)| match board.start() {
                        Ok(board) => (pid, board),
                        Err(_) => unreachable!(),
                    })
                    .collect(),
                turn_order: self.turn_order,
                current: 0,
            })
        }
    }

    /// Add a player to the game, specifying their ID and the dimensions of their board.
    pub fn add_player(
        &mut self,
        pid: P,
        dim: D,
    ) -> Result<&mut BoardSetup<I, D, S>, AddPlayerError<P, D>> {
        match self.boards.entry(pid.clone()) {
            Entry::Occupied(_) => Err(AddPlayerError::new(pid, dim)),
            Entry::Vacant(entry) => {
                self.turn_order.push(pid);
                Ok(entry.insert(BoardSetup::new(dim)))
            }
        }
    }

    /// Checks if at least two players have been added to the game and all players are
    /// ready
    pub fn ready(&self) -> bool {
        self.boards.len() >= 2 && self.boards.values().all(|board| board.ready())
    }

    /// Get the board for the player with the specified ID.
    pub fn get_board<Q: ?Sized>(&self, pid: &Q) -> Option<&BoardSetup<I, D, S>>
    where
        P: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.boards.get(pid)
    }

    /// Mutably get the board for the player with the specified ID.
    pub fn get_board_mut<Q: ?Sized>(&mut self, pid: &Q) -> Option<&mut BoardSetup<I, D, S>>
    where
        P: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.boards.get_mut(pid)
    }
}

impl<P: PlayerId, I: ShipId, D: Dimensions, S: ShipShape<D>> Default for GameSetup<P, I, D, S> {
    fn default() -> Self {
        Self::new()
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
    /// sunk. However, there are additonal players left who still have ships.
    Defeated(I),
    /// The shot hit the ship with the given ID and all players but the current player are
    /// now defeated. The current player is the winner.
    Victory(I),
}

impl<I> ShotOutcome<I> {
    /// Get the id of the ship that was hit.
    pub fn ship(&self) -> Option<&I> {
        match self {
            ShotOutcome::Miss => None,
            ShotOutcome::Hit(ref id)
            | ShotOutcome::Sunk(ref id)
            | ShotOutcome::Defeated(ref id)
            | ShotOutcome::Victory(ref id) => Some(id),
        }
    }

    /// Extract the id of the ship that was hit from this result.
    pub fn into_ship(self) -> Option<I> {
        match self {
            ShotOutcome::Miss => None,
            ShotOutcome::Hit(id)
            | ShotOutcome::Sunk(id)
            | ShotOutcome::Defeated(id)
            | ShotOutcome::Victory(id) => Some(id),
        }
    }
}

impl<I> From<BoardShotOutcome<I>> for ShotOutcome<I> {
    fn from(shot: BoardShotOutcome<I>) -> Self {
        match shot {
            BoardShotOutcome::Miss => ShotOutcome::Miss,
            BoardShotOutcome::Hit(id) => ShotOutcome::Hit(id),
            BoardShotOutcome::Sunk(id) => ShotOutcome::Sunk(id),
            BoardShotOutcome::Defeated(id) => ShotOutcome::Defeated(id),
        }
    }
}

/// Handles gameplay.
pub struct Game<P: PlayerId, I: ShipId, D: Dimensions> {
    /// Gameplay boards for the players.
    boards: HashMap<P, Board<I, D>>,

    /// Records the turn order for players.
    turn_order: Vec<P>,

    /// Counter for the current player turn as an index in `turn_order`.
    current: usize,
}

impl<P: PlayerId, I: ShipId, D: Dimensions> Game<P, I, D> {
    /// Get the ID of the player whose turn it is.
    pub fn current(&self) -> &P {
        &self.turn_order[self.current]
    }

    /// Get the status of the game. Returns `None` if the game is in progress, otherwise
    /// returns the winner.
    pub fn winner(&self) -> Option<&P> {
        let remaining = self
            .boards
            .values()
            .filter(|board| !board.defeated())
            .count();
        debug_assert!(remaining > 0);
        if remaining == 1 {
            Some(self.current())
        } else {
            None
        }
    }

    /// Get a reference to the board for the specified player.
    pub fn get_board<Q: ?Sized>(&self, pid: &Q) -> Option<&Board<I, D>>
    where
        P: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.boards.get(pid)
    }

    /// Iterate the player ids and boards in turn-order.
    pub fn iter_boards(&self) -> impl Iterator<Item = (&P, &Board<I, D>)> {
        self.turn_order
            .iter()
            .map(move |pid| (pid, &self.boards[pid]))
    }

    /// Fire a shot at the specified player, returning the result of the shot or
    /// an error if the shot was invalid.
    pub fn shoot(
        &mut self,
        target: P,
        coord: D::Coordinate,
    ) -> Result<ShotOutcome<I>, ShotError<P, D::Coordinate>> {
        if self.winner().is_some() {
            Err(ShotError::new(
                CannotShootReason::AlreadyOver,
                target,
                coord,
            ))
        } else if self.current() == &target {
            Err(ShotError::new(CannotShootReason::SelfShot, target, coord))
        } else if let Some(board) = self.boards.get_mut(&target) {
            match board.shoot(coord) {
                Ok(BoardShotOutcome::Defeated(id)) if self.winner().is_some() => {
                    Ok(ShotOutcome::Victory(id))
                }
                Ok(res) => Ok(res.into()),
                Err(err) => Err(ShotError::add_context(err, target)),
            }
        } else {
            Err(ShotError::new(
                CannotShootReason::UnknownPlayer,
                target,
                coord,
            ))
        }
    }
}
