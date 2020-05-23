//! Implementation of the game as whole. This is split into different modules for
//! different complexity levels.
//!
//! [`simple`] provides an implementation of the simplest form of the game: 10x10 grid
//! with two players and the same set of sandard ships for both players. This is a simple
//! wrapper around the [`uniform`] game.
//!
//! [`uniform`] provides an implementation that allows a fair amount of flexibility in
//! terms of the number of players, the ships available to each player, and the exact
//! dimensions of each player's board, but requires uniform generic arguments for all
//! players.
//!
//! [`dynamic`] provides support for fully-dynamic games where every player might be
//! playing on a completely different board type with different ships and coordinate
//! formats.

pub mod simple;
pub mod uniform;
pub mod dynamic {
    //! Not yet implemented.
}
