use std::{fmt::Debug, hash::Hash};

/// Trait for coordinates used in [`Dimensions`].
/// Requires [`Debug`] to enable certain common panic messages on misuse.
/// Coordinates are treated as disposable and cheaply cloneable. If you need a complex
/// coordinate type that isn't cheap to clone, you may want to wrap it in `Rc` or `Arc`.
pub trait Coordinate: Debug + Clone + Eq + Hash {}

/// Dimensions of a board.
/// Implements methods needed for the board to check bounds, linearize indexes, and compute
/// neighbor cells.
pub trait Dimensions: Debug {
    /// The type used to identify cells on the board.
    type Coordinate: Coordinate;

    /// Type used in the neighbor iterator.
    type NeighborIterState: NeighborIterState<Dimensions = Self>;

    /// Compute the total size of the dimensions. Used to allocate storage for the board.
    fn total_size(&self) -> usize;

    /// Convert a coordinate to a linear index within this dimension.
    /// Panics if the coordinate is out of range for the dimension.
    fn linearize(&self, coord: &Self::Coordinate) -> usize {
        match self.try_linearize(coord) {
            Some(v) => v,
            None => panic!("{:?} is out of bounds for {:?}", coord, self),
        }
    }

    /// Convert a coordinate to a linear index within this dimension.
    /// Returns `None` if the coordinate is out of bound for the dimension.
    fn try_linearize(&self, coord: &Self::Coordinate) -> Option<usize>;

    /// Get back a coordinate from a linearized index. Panic if idx is >= total_size.
    fn un_linearize(&self, idx: usize) -> Self::Coordinate;

    /// Iterate the neighbors of the given coordinate.
    fn neighbors(&self, coord: Self::Coordinate) -> NeighborIter<Self::NeighborIterState> {
        NeighborIter {
            dim: self,
            state: Self::NeighborIterState::start(self, coord),
        }
    }

    /// Return true if the given coordinates are neighbors. Defeault implemntation checks
    /// the neighbors iter. A board may wish to provide a more efficient implementation.
    fn is_neighbor(&self, c1: &Self::Coordinate, c2: &Self::Coordinate) -> bool {
        self.neighbors(c1.clone()).any(|n| &n == c2)
    }
}

/// Trait for [`Dimensions`] that support colinearity checks on their coordinates.
pub trait ColinearCheck: Dimensions {
    /// Returns true if the 3 coordinates are colinear.
    fn is_colinear(
        &self,
        c1: &Self::Coordinate,
        c2: &Self::Coordinate,
        c3: &Self::Coordinate,
    ) -> bool;
}

/// State type for the neighbor iterator.
pub trait NeighborIterState {
    type Dimensions: Dimensions + ?Sized;

    /// Construct an instance of this iter state given the arguments.
    fn start(dim: &Self::Dimensions, coord: <Self::Dimensions as Dimensions>::Coordinate) -> Self;

    /// Get the next item given a reference to the parent type.
    fn next(
        &mut self,
        dim: &Self::Dimensions,
    ) -> Option<<Self::Dimensions as Dimensions>::Coordinate>;
}

/// Iterator over the neighbors of a coordinate.
pub struct NeighborIter<'a, S: NeighborIterState> {
    dim: &'a S::Dimensions,
    state: S,
}

impl<'a, S: NeighborIterState> Iterator for NeighborIter<'a, S> {
    type Item = <S::Dimensions as Dimensions>::Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next(self.dim)
    }
}
