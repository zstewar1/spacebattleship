//! Types used for defining ships and their shapes.
use std::{fmt::Debug, hash::Hash};

use crate::board::Dimensions;

pub use self::linear::Line;

mod linear;

/// Trait for types that can be used as a Ship's ID within a single player's board.
/// IDs are treated as disposable and cheaply cloneable. If you need a complex ID type
/// that isn't cheap to clone, you may want to wrap it in `Rc` or `Arc`.
///
/// Auto-implemented for any type which implements `Debug`,`Clone`, `Eq`, and `Hash`.
pub trait ShipId: Debug + Clone + Eq + Hash {}
impl<T: Debug + Clone + Eq + Hash> ShipId for T {}

/// Trait for shapes that a ship can be.
pub trait ShipShape<D: Dimensions + ?Sized> {
    type ProjectIterState: ProjectIterState<D, ShipShape = Self>;

    /// Get an iterator over possible placements of this ship shap in the given
    /// dimensions. Does not in any way account for whether cells are already occupied or
    /// not.
    fn project<'a>(
        &'a self,
        coord: D::Coordinate,
        dim: &'a D,
    ) -> ProjectIter<D, Self::ProjectIterState> {
        ProjectIter {
            shape: self,
            dim: dim,
            state: Self::ProjectIterState::start(self, dim, coord),
        }
    }

    /// Return true if the given shape projection is a valid placement of this ship in the
    /// specified dimensions. Does not account for whether cells are already occupied.
    /// Shapes are free to reject any placement that they did not generate.
    fn is_valid_placement(&self, proj: &ShapeProjection<D::Coordinate>, dim: &D) -> bool;
}

/// Projection of a shape onto a coordinate system relative to a particular point. This is
/// a simple typedef of a `Vec`, however projections retrieved from a particular Ship-
/// Shape should not be modified, as shapes are free to reject any projection that they
/// did not generate.
pub type ShapeProjection<C> = Vec<C>;

/// State type for the ship projection iterator.
pub trait ProjectIterState<D: Dimensions + ?Sized> {
    type ShipShape: ShipShape<D> + ?Sized;

    /// Construct an instance of this iter state given the arguments.
    fn start(shape: &Self::ShipShape, dim: &D, coord: D::Coordinate) -> Self;

    /// Get the next possible projection of the ship's shape.
    fn next(&mut self, shape: &Self::ShipShape, dim: &D) -> Option<ShapeProjection<D::Coordinate>>;
}

/// Iterator over possible projections of a ship shape onto dimensions.
pub struct ProjectIter<'a, D, S>
where
    D: Dimensions + ?Sized,
    S: ProjectIterState<D>,
{
    shape: &'a S::ShipShape,
    dim: &'a D,
    state: S,
}

impl<'a, D, S> Iterator for ProjectIter<'a, D, S>
where
    D: Dimensions + ?Sized,
    S: ProjectIterState<D>,
{
    type Item = ShapeProjection<D::Coordinate>;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next(self.shape, self.dim)
    }
}
