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

use crate::board::Coordinate;

/// The corrdinates of a [`GridCell`][crate::board::GridCell] in the board.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coordinate2D {
    /// Horizontal position of the cell.
    pub x: usize,
    /// Vertical position of the cell.
    pub y: usize,
}

impl Coordinate2D {
    /// Construct a [`Coordinate2D`] from the given `x` and `y`.
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Coordinate for Coordinate2D {}

impl From<(usize, usize)> for Coordinate2D {
    /// Construct a [`Coordinate2D`] from the given `(x, y)` pair.
    fn from((x, y): (usize, usize)) -> Self {
        Self::new(x, y)
    }
}

impl From<Coordinate2D> for (usize, usize) {
    /// Convert the [`Coordinate2D`] into an `(x, y)` pair.
    fn from(coord: Coordinate2D) -> Self {
        (coord.x, coord.y)
    }
}

#[cfg(feature = "rng_gen")]
pub use rand_impl::UniformCoordinate2D;

#[cfg(feature = "rng_gen")]
mod rand_impl {
    use super::Coordinate2D;
    use rand::{
        distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler},
        Rng,
    };

    impl SampleUniform for Coordinate2D {
        type Sampler = UniformCoordinate2D;
    }

    /// Implements uniform sampling for [`Coordinate2D`].
    pub struct UniformCoordinate2D(UniformInt<usize>, UniformInt<usize>);

    impl UniformSampler for UniformCoordinate2D {
        type X = Coordinate2D;

        fn new<B1, B2>(low: B1, high: B2) -> Self
        where
            B1: SampleBorrow<Self::X> + Sized,
            B2: SampleBorrow<Self::X> + Sized,
        {
            UniformCoordinate2D(
                UniformInt::new(low.borrow().x, high.borrow().x),
                UniformInt::new(low.borrow().y, high.borrow().y),
            )
        }
        fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
        where
            B1: SampleBorrow<Self::X> + Sized,
            B2: SampleBorrow<Self::X> + Sized,
        {
            UniformCoordinate2D(
                UniformInt::new_inclusive(low.borrow().x, high.borrow().x),
                UniformInt::new_inclusive(low.borrow().y, high.borrow().y),
            )
        }
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
            Coordinate2D {
                x: self.0.sample(rng),
                y: self.1.sample(rng),
            }
        }
    }
}
