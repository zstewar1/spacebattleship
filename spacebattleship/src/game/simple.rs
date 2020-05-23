//! Implementation of the basic game of battleship with two players and five ships on a
//! 10x10 grid.
use std::{cmp::Ordering, ops::Deref};

use thiserror::Error;

use crate::{
    board::{
        rectangular::{Coordinate, RectDimensions},
        BoardSetup, self,
    },
    game::uniform,
    ships::{Line, ShapeProjection},
};

/// Alias to ShipRef with fixed generic types.
pub type ShipRef<'a> = board::ShipRef<'a, Ship, RectDimensions>;
/// Alias to CellRef with fixed generic types.
pub type CellRef<'a> = board::CellRef<'a, Ship, RectDimensions>;

/// Player ID for the simple game. Either `P1` or `P2`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    /// Get the oponent of this player.
    pub fn oponent(self) -> Self {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

/// Ship ID for the simple game.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Ship {
    /// Carrier: length 5.
    Carrier,
    /// Battleship: length 4.
    Battleship,
    /// Cruiser: length 3.
    Cruiser,
    /// Submarine: length 3.
    Submarine,
    /// Destroyer: length 2.
    Destroyer,
}

impl Ship {
    /// Get the shape cooresponding to this ship ID.
    fn get_shape(self) -> Line {
        Line::new(self.len())
    }

    /// Get the length of this ship type.
    pub fn len(self) -> usize {
        match self {
            Ship::Carrier => 5,
            Ship::Battleship => 4,
            Ship::Cruiser => 3,
            Ship::Submarine => 3,
            Ship::Destroyer => 2,
        }
    }
}

/// Reason why a ship could not be placed at a given position.
#[derive(Debug, Error, Copy, Clone, Eq, PartialEq)]
pub enum CannotPlaceReason {
    /// The ship did not fit in the given direction.
    #[error("insufficient space for the ship at the specified position")]
    InsufficientSpace,
    /// The ship was already placed.
    #[error("specified ship was already placed")]
    AlreadyPlaced,
    /// The space selected overlaps a ship that was already placed.
    #[error("the specified position was already occupied")]
    AlreadyOccupied,
}

/// Placement orientation of a ship.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

impl Orientation {
    /// Check if the given projection is pointed along this orientation.
    fn check_dir(self, proj: &ShapeProjection<Coordinate>) -> bool {
        if proj.len() < 2 {
            // None of the current ships should have len 1, but support it here anyway.
            true
        } else {
            let dx = proj[0].x.cmp(&proj[1].x);
            let dy = proj[0].y.cmp(&proj[1].y);
            match (self, dx, dy) {
                (Orientation::Up, Ordering::Equal, Ordering::Less) => true,
                (Orientation::Down, Ordering::Equal, Ordering::Greater) => true,
                (Orientation::Left, Ordering::Less, Ordering::Equal) => true,
                (Orientation::Right, Ordering::Greater, Ordering::Equal) => true,
                _ => false,
            }
        }
    }
}

/// Represents a placement of a ship. Allows extracting the orientation and start, as well
/// as iterating the coordinates.
pub struct Placement([Coordinate]);

impl Placement {
    fn from_coords(coords: &[Coordinate]) -> &Placement {
        unsafe { std::mem::transmute(coords) }
    }

    pub fn orientation(&self) -> Orientation {
        if self.len() < 2 {
            // None of the current ships are less than 2 len, but we can handle it anyway.
            Orientation::Up
        } else {
            let dx = self[0].x.cmp(&self[1].x);
            let dy = self[0].y.cmp(&self[1].y);
            match (dx, dy) {
                (Ordering::Equal, Ordering::Less) => Orientation::Up,
                (Ordering::Equal, Ordering::Greater) => Orientation::Down,
                (Ordering::Less, Ordering::Equal) => Orientation::Left,
                (Ordering::Greater, Ordering::Equal) => Orientation::Right,
                // Shouldn't happen since we don't allow building paths that don't follow
                // these rules.
                _ => panic!("Coordinates don't point along a valid orientation"),
            }
        }
    }

    /// Get the coordinate where this placement starts.
    pub fn start(&self) -> &Coordinate {
        // This will panic if len is 0. That's OK because this type has no public
        // constructor and we know that within this module we never create placements with
        // 0 length.
        &self[0]
    }
}

impl Deref for Placement {
    type Target = [Coordinate];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Struct used to setup the simple game.
pub struct GameSetup(uniform::GameSetup<Player, Ship, RectDimensions, Line>);

impl GameSetup {
    /// Create a [`GameSetup`] for the game, including two players with one of each ship.
    pub fn new() -> Self {
        let mut setup = uniform::GameSetup::new();
        Self::add_ships(
            setup
                .add_player(Player::P1, RectDimensions::new(10, 10))
                .unwrap(),
        );
        Self::add_ships(
            setup
                .add_player(Player::P2, RectDimensions::new(10, 10))
                .unwrap(),
        );
        GameSetup(setup)
    }

    /// Add the initial ships for the player.
    fn add_ships(board: &mut BoardSetup<Ship, RectDimensions, Line>) {
        Self::add_ship(Ship::Carrier, board);
        Self::add_ship(Ship::Battleship, board);
        Self::add_ship(Ship::Cruiser, board);
        Self::add_ship(Ship::Submarine, board);
        Self::add_ship(Ship::Destroyer, board);
    }

    /// Add the given ship to the board.
    fn add_ship(ship: Ship, board: &mut BoardSetup<Ship, RectDimensions, Line>) {
        board.add_ship(ship, ship.get_shape()).unwrap();
    }

    /// Tries to start the game. If all players are ready, returns a [`Game`], otherwise
    /// returns self.
    pub fn start(self) -> Result<Game, Self> {
        match self.0.start() {
            Ok(game) => Ok(Game(game)),
            Err(setup) => Err(GameSetup(setup)),
        }
    }

    /// Return true if both players are ready to start the game.
    pub fn ready(&self) -> bool {
        self.0.ready()
    }

    /// Check if the specified player is ready.
    pub fn is_player_ready(&self, player: Player) -> bool {
        self.0.get_board(&player).unwrap().ready()
    }

    /// Get an iterator over all the ship IDs for the given player and the coordinates
    /// where that ship is placed, if any.
    pub fn get_ships<'a>(
        &'a self,
        player: Player,
    ) -> impl 'a + Iterator<Item = (Ship, Option<&'a Placement>)> {
        self.0.get_board(&player).unwrap().iter_ships().map(|ship| {
            (
                *ship.id(),
                ship.placement().map(|v| Placement::from_coords(v)),
            )
        })
    }

    /// Get the ships for the specified player which still need to be placed.
    pub fn get_pending_ships<'a>(&'a self, player: Player) -> impl 'a + Iterator<Item = Ship> {
        self.get_ships(player)
            .filter_map(|(ship, placement)| match placement {
                Some(_) => None,
                None => Some(ship),
            })
    }

    /// Get the the coordinates where the given ship is placed, if any.
    pub fn get_placement(&self, player: Player, ship: Ship) -> Option<&Placement> {
        self.0
            .get_board(&player)
            .unwrap()
            .get_ship(ship)
            .unwrap()
            .placement()
            .map(|v| Placement::from_coords(v))
    }

    /// Check if the given placement would be valid, without attempting to actually place
    /// the ship.
    pub fn check_placement(
        &self,
        player: Player,
        ship: Ship,
        start: Coordinate,
        dir: Orientation,
    ) -> Result<(), CannotPlaceReason> {
        let board = self.0.get_board(&player).unwrap();
        let ship = board.get_ship(ship).unwrap();
        let proj = ship
            .get_placements(start)
            .find(|proj| dir.check_dir(proj))
            .ok_or(CannotPlaceReason::InsufficientSpace)?;
        ship.check_placement(&proj).map_err(|err| match err {
            board::CannotPlaceReason::AlreadyOccupied => CannotPlaceReason::AlreadyOccupied,
            board::CannotPlaceReason::AlreadyPlaced => CannotPlaceReason::AlreadyPlaced,
            // We will never provide an invalid projection.
            board::CannotPlaceReason::InvalidProjection => unreachable!(),
        })
    }

    /// Try to place the specified ship at the specified position, returning an
    /// error if placement is not possible.
    pub fn place_ship(
        &mut self,
        player: Player,
        ship: Ship,
        start: Coordinate,
        dir: Orientation,
    ) -> Result<(), CannotPlaceReason> {
        let board = self.0.get_board_mut(&player).unwrap();
        let mut ship = board.get_ship_mut(ship).unwrap();
        let proj = ship
            .get_placements(start)
            .find(|proj| dir.check_dir(proj))
            .ok_or(CannotPlaceReason::InsufficientSpace)?;
        ship.place(proj).map_err(|err| match err.reason() {
            board::CannotPlaceReason::AlreadyOccupied => CannotPlaceReason::AlreadyOccupied,
            board::CannotPlaceReason::AlreadyPlaced => CannotPlaceReason::AlreadyPlaced,
            // We will never provide an invalid projection.
            board::CannotPlaceReason::InvalidProjection => unreachable!(),
        })
    }

    /// Clear the placement of the specified ship. Return true if the ship was previously
    /// placed.
    pub fn unplace_ship(&mut self, player: Player, ship: Ship) -> bool {
        self.0
            .get_board_mut(&player)
            .unwrap()
            .get_ship_mut(ship)
            .unwrap()
            .unplace()
            .is_some()
    }

    /// Get an iterator over the specified player's board. The iterator's item is another
    /// iterator that iterates over a single row.
    pub fn iter_board<'a>(
        &'a self,
        player: Player,
    ) -> impl 'a + Iterator<Item = impl 'a + Iterator<Item = Option<Ship>>> {
        let board = self.0.get_board(&player).unwrap();
        board
            .dimensions()
            .iter_coordinates()
            .map(move |row| row.map(move |coord| board.get_coord(&coord).copied()))
    }
}

/// Reason why a shot at the board failed.
#[derive(Debug, Error, Copy, Clone, Eq, PartialEq)]
pub enum CannotShootReason {
    /// The game is already over
    #[error("the game is already over")]
    AlreadyOver,

    /// The target player is the player whose turn it is.
    #[error("player attempted to shoot out of turn")]
    OutOfTurn,

    /// The specified cell is out of bounds for the grid.
    #[error("the target coordinate is out of bounds")]
    OutOfBounds,

    /// The specified cell has already been shot.
    #[error("the target cell was already shot")]
    AlreadyShot,
}

/// Outcome of a successfully-fired shot.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShotOutcome {
    /// Nothing was hit.
    Miss,
    /// The given ship was hit but it was not sunk.
    Hit(Ship),
    /// The given ship was hit and it was sunk but the player still had other ships.
    Sunk(Ship),
    /// The given ship was hit and sunk, and the target player has no remaining ships.
    Victory(Ship),
}

/// Simplified game that uses a fixed set of ships and players.
pub struct Game(uniform::Game<Player, Ship, RectDimensions>);

impl Game {
    /// Get the player whose turn it currently is.
    pub fn current(&self) -> Player {
        *self.0.current()
    }

    /// Get the status of the game. Returns `None` if the game is in progress, otherwise
    /// returns the winner.
    pub fn winner(&self) -> Option<Player> {
        self.0.winner().copied()
    }

    /// Get an iterator over the specified player's board. The iterator's item is another
    /// iterator that iterates over a single row.
    pub fn iter_board<'a>(
        &'a self,
        player: Player,
    ) -> impl 'a + Iterator<Item = impl 'a + Iterator<Item = CellRef<'a>>> {
        let board = self.0.get_board(&player).unwrap();
        board
            .dimensions()
            .iter_coordinates()
            .map(move |row| row.map(move |coord| board.get_coord(coord).unwrap()))
    }

    /// Get an iterator over the specified player's ships.
    pub fn iter_ships<'a>(&'a self, player: Player) -> impl 'a + Iterator<Item = ShipRef<'a>> {
        self.0.get_board(&player).unwrap().iter_ships()
    }

    /// Get a reference to the cell with the specified coordinate in the specified 
    /// player's board. Return None if the coord is out of bounds.
    pub fn get_coord(&self, player: Player, coord: Coordinate) -> Option<CellRef> {
        self.0.get_board(&player).unwrap().get_coord(coord)
    }

    /// Get a reference to the specified ship from the specified player's board.
    pub fn get_ship(&self, player: Player, ship: Ship) -> ShipRef {
        self.0.get_board(&player).unwrap().get_ship(&ship).unwrap()
    }

    /// Fire at the specified player on the specified coordinate.
    pub fn shoot(&mut self, target: Player, coord: Coordinate) -> Result<ShotOutcome, CannotShootReason> {
        self.0.shoot(target, coord).map(|outcome| match outcome {
            uniform::ShotOutcome::Miss => ShotOutcome::Miss,
            uniform::ShotOutcome::Hit(ship) => ShotOutcome::Hit(ship),
            uniform::ShotOutcome::Sunk(ship) => ShotOutcome::Sunk(ship),
            // There are only two players so if one is defeated, we should go directly to
            // victory and never hit Defeated.
            uniform::ShotOutcome::Defeated(_) => unreachable!(),
            uniform::ShotOutcome::Victory(ship) => ShotOutcome::Victory(ship),
        }).map_err(|err| match err.reason() {
            uniform::CannotShootReason::AlreadyOver => CannotShootReason::AlreadyOver,
            uniform::CannotShootReason::SelfShot => CannotShootReason::OutOfTurn,
            // There are always exactly two players, so player will never be unknown.
            uniform::CannotShootReason::UnknownPlayer => unreachable!(),
            // Since there are only 2 players, if one is defeated, the reason will be 
            // AlreadyOver not AlreadyDefeated
            uniform::CannotShootReason::AlreadyDefeated => unreachable!(),
            uniform::CannotShootReason::OutOfBounds => CannotShootReason::OutOfBounds,
            uniform::CannotShootReason::AlreadyShot => CannotShootReason::AlreadyShot,
        })
    }
}