//! Defines the types that make up the grid. These are shared between the board's setup
//! and playing versions.

use std::{
    borrow::Borrow,
    ops::{Index, IndexMut},
};

use crate::board::Dimensions;

/// A single cell in the player's grid.
#[derive(Debug)]
pub(super) struct GridCell<I> {
    /// The ID of the ship that occupies this cell, if any.
    pub(super) ship: Option<I>,

    /// Whether this cell has been hit previously or not.
    pub(super) hit: bool,
}

impl<I> Default for GridCell<I> {
    fn default() -> Self {
        Self {
            ship: None,
            hit: false,
        }
    }
}

/// Grid structure shared between [`BoardSetup`] and [`Board`].
#[derive(Debug)]
pub(super) struct Grid<I, D> {
    /// Dimensions of this board.
    pub(super) dim: D,
    /// Cells that make up this board.
    pub(super) cells: Box<[GridCell<I>]>,
}

impl<I, D: Dimensions> Grid<I, D> {
    pub(super) fn new(dim: D) -> Self {
        let cells = (0..dim.total_size()).map(|_| Default::default()).collect();
        Self { dim, cells }
    }

    /// Get a reference to the cell at the given [`Coordinate`].
    pub(super) fn get<B: Borrow<D::Coordinate>>(&self, coord: B) -> Option<&GridCell<I>> {
        self.dim
            .try_linearize(coord.borrow())
            .and_then(|i| self.cells.get(i))
    }

    /// Get a mutable reference to the cell at the given [`Coordinate`].
    pub(super) fn get_mut<B: Borrow<D::Coordinate>>(
        &mut self,
        coord: B,
    ) -> Option<&mut GridCell<I>> {
        self.dim
            .try_linearize(coord.borrow())
            .and_then(move |i| self.cells.get_mut(i))
    }
}

impl<I, D: Dimensions, B: Borrow<D::Coordinate>> Index<B> for Grid<I, D> {
    type Output = GridCell<I>;

    fn index(&self, coord: B) -> &Self::Output {
        self.get(coord).expect("coordinate out of bounds")
    }
}

impl<I, D: Dimensions, B: Borrow<D::Coordinate>> IndexMut<B> for Grid<I, D> {
    fn index_mut(&mut self, coord: B) -> &mut Self::Output {
        self.get_mut(coord).expect("coordinate out of bounds")
    }
}
