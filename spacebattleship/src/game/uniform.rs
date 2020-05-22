//! Implementation of Battleship with uniform generic parameters, if not uniform board
//! setups.
use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::Hash,
};

use crate::{
    board::{Board, BoardSetup, Dimensions},
    ships::{ShipId, ShipShape},
};

pub use self::errors::AddPlayerError;

mod errors;

/// Types used for the ID of a player. Ids may be cloned arbitrarily so they should be
/// cheap to clone.
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

/// Handles gameplay.
pub struct Game<P: PlayerId, I: ShipId, D: Dimensions> {
    /// Gameplay boards for the players.
    boards: HashMap<P, Board<I, D>>,

    /// Records the turn order for players.
    turn_order: Vec<P>,

    /// Counter for the current player turn as an index in `turn_order`.
    current: usize,
}
