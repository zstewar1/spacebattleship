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
