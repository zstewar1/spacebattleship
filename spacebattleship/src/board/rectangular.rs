//! Implements a basic rectangular board.
use std::borrow::Borrow;

use enumflags2::BitFlags;

use crate::board::{ColinearCheck, Dimensions, NeighborIterState};

pub use crate::board::common::Coordinate2D as Coordinate;

/// Controls which dimensions the grid wraps around in.
#[derive(BitFlags, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Wrapping {
    /// The grid wraps along the `x` direction.
    Horizontal = 0b01,
    /// The grid wraps along the `y` direction.
    Vertical = 0b10,
}

/// Simple rectangular dimensions. Optionally supports wrapping.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RectDimensions {
    /// Width of the board. This cooresponds to the `x` [`Coordinate`].
    width: usize,
    /// Height of the board. This cooresponds to the `y` [`Coordinate`].
    height: usize,

    /// Set of orientations that the grid wraps along.
    wrapping: BitFlags<Wrapping>,
}

impl RectDimensions {
    /// Create new [`RectDimensions`] with the specified width and height. Defaults to no wrapping.
    /// Panics if `width * height` exceeds `usize::max_value()` or if `width` or `height` is 0.
    pub fn new(width: usize, height: usize) -> Self {
        Self::new_wrapping(width, height, BitFlags::empty())
    }

    /// Create new [`RectDimensions`] with the specified width and height, wrapping on the
    /// specified axes.
    /// Panics if `width * height` exceeds `usize::max_value()` or if `width` or `height` is 0.
    pub fn new_wrapping<B: Into<BitFlags<Wrapping>>>(
        width: usize,
        height: usize,
        wrapping: B,
    ) -> Self {
        match Self::try_new_wrapping(width, height, wrapping) {
            Some(dim) => dim,
            None => {
                if width == 0 || height == 0 {
                    panic!("RectDimensions must be nonzero, got {}x{}", width, height);
                } else {
                    panic!(
                        "RectDimesnsions too large: {} * {} > {}",
                        width,
                        height,
                        usize::max_value()
                    );
                }
            }
        }
    }

    /// Create new [`RectDimensions`] with the specified width and height. Defaults to no wrapping.
    /// Returns `None` if `width * height` exceeds `usize::max_value()` or if `width` or `height`
    /// is 0.
    pub fn try_new(width: usize, height: usize) -> Option<Self> {
        Self::try_new_wrapping(width, height, BitFlags::empty())
    }

    /// Create new [`RectDimensions`] with the specified width and height.
    /// Returns `None` if `width * height` exceeds `usize::max_value()` or if `width` or `height`
    /// is 0.
    pub fn try_new_wrapping<B: Into<BitFlags<Wrapping>>>(
        width: usize,
        height: usize,
        wrapping: B,
    ) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            width.checked_mul(height).map(|_| Self {
                width,
                height,
                wrapping: wrapping.into(),
            })
        }
    }

    /// Get the width of these [`RectDimensions`].
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of these [`RectDimensions`].
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the wrapping modes of these [`RectDimensions`].
    pub fn wrapping(&self) -> BitFlags<Wrapping> {
        self.wrapping
    }

    /// Whether the grid wraps along the `x` direciton.
    pub fn wrap_x(&self) -> bool {
        self.wrapping.contains(Wrapping::Horizontal)
    }

    /// Whether the grid wraps along the `y` direciton.
    pub fn wrap_y(&self) -> bool {
        self.wrapping.contains(Wrapping::Vertical)
    }

    /// Get an iterator over rows of this grid. Each row is an iterator over the coordinates of
    /// that row.
    pub fn iter_coordinates(&self) -> impl Iterator<Item = impl Iterator<Item = Coordinate>> {
        let width = self.width;
        (0..self.height).map(move |y| (0..width).map(move |x| Coordinate { x, y }))
    }

    /// Check if the given [`Coordinate`] is in bounds for these [`RectDimensions`]. If so, return
    /// it, otherwise return `None`.
    #[inline]
    fn check_bounds<B: Borrow<Coordinate>>(&self, coord: B) -> Option<B> {
        let c = coord.borrow();
        if c.x < self.width && c.y < self.height {
            Some(coord)
        } else {
            None
        }
    }
}

impl Dimensions for RectDimensions {
    type Coordinate = Coordinate;

    type NeighborIterState = RectNeighbors;

    /// Compute the linear total size of these [`Dimensions`].
    fn total_size(&self) -> usize {
        self.width * self.height
    }

    /// Convert a coordinate to a linear index within this dimension.
    /// Returns `None` if the coordinate is out of range for the dimension.
    fn try_linearize(&self, coord: &Self::Coordinate) -> Option<usize> {
        self.check_bounds(coord)
            .map(|coord| coord.y * self.width + coord.x)
    }

    /// Convert a linear index back into a [`Coordinate`].
    fn un_linearize(&self, idx: usize) -> Coordinate {
        Coordinate {
            x: idx % self.width,
            y: idx / self.width,
        }
    }
}

impl ColinearCheck for RectDimensions {
    fn is_colinear(&self, c1: &Coordinate, c2: &Coordinate, c3: &Coordinate) -> bool {
        let difx = c1.x != c2.x || c2.x != c3.x;
        let dify = c1.y != c2.y || c2.y != c3.y;
        // Allowed to differ in only one direction.
        !(difx && dify)
    }
}

impl Default for RectDimensions {
    /// Construct the default rectangular dimensions, a 10x10 board with no wrapping.
    fn default() -> Self {
        Self {
            width: 10,
            height: 10,
            wrapping: BitFlags::empty(),
        }
    }
}

/// State of the neighbors iter for RectDimensions.
pub struct RectNeighbors {
    coord: Coordinate,
    step: RectNeighborsStep,
}

#[derive(Debug, Copy, Clone)]
enum RectNeighborsStep {
    Up,
    Down,
    Left,
    Right,
    End,
}

impl NeighborIterState for RectNeighbors {
    type Dimensions = RectDimensions;

    fn start(dim: &RectDimensions, coord: Coordinate) -> Self {
        Self {
            coord,
            // If the coordinate is out of bounds, skip directly to the End state so we
            // don't have to run dim.check_bounds every iteration.
            step: dim
                .check_bounds(coord)
                .map_or(RectNeighborsStep::End, |_| RectNeighborsStep::Up),
        }
    }

    fn next(&mut self, dim: &RectDimensions) -> Option<Coordinate> {
        loop {
            match self.step {
                RectNeighborsStep::Up => {
                    self.step = RectNeighborsStep::Down;
                    match self.coord.y.checked_sub(1) {
                        Some(y) => return Some(Coordinate::new(self.coord.x, y)),
                        None if dim.wrap_y() => {
                            return Some(Coordinate::new(self.coord.x, dim.height - 1))
                        }
                        None => {}
                    }
                }
                RectNeighborsStep::Down => {
                    self.step = RectNeighborsStep::Left;
                    match self.coord.y + 1 {
                        y if y < dim.height => return Some(Coordinate::new(self.coord.x, y)),
                        _ if dim.wrap_y() => return Some(Coordinate::new(self.coord.x, 0)),
                        _ => {}
                    }
                }
                RectNeighborsStep::Left => {
                    self.step = RectNeighborsStep::Right;
                    match self.coord.x.checked_sub(1) {
                        Some(x) => return Some(Coordinate::new(x, self.coord.y)),
                        None if dim.wrap_x() => {
                            return Some(Coordinate::new(dim.width - 1, self.coord.y))
                        }
                        None => {}
                    }
                }
                RectNeighborsStep::Right => {
                    self.step = RectNeighborsStep::End;
                    match self.coord.x + 1 {
                        x if x < dim.width => return Some(Coordinate::new(x, self.coord.y)),
                        _ if dim.wrap_x() => return Some(Coordinate::new(0, self.coord.y)),
                        _ => {}
                    }
                }
                RectNeighborsStep::End => return None,
            }
        }
    }
}
