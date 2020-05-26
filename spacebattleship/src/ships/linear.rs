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
use std::collections::HashSet;

use crate::{
    board::{ColinearCheck, Dimensions},
    ships::{ProjectIterState, ShapeProjection, ShipShape},
};

/// A linear ship shape, with a given length.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Line(usize);

impl Line {
    /// Construct a linear ship with the specified length. Panics if len is 0.
    pub fn new(len: usize) -> Self {
        assert!(len > 0);
        Line(len)
    }

    /// Get the length of this ship.
    pub fn len(&self) -> usize {
        self.0
    }
}

impl<D: Dimensions + ColinearCheck + ?Sized> ShipShape<D> for Line {
    type ProjectIterState = LineProjectIterState<D::Coordinate>;

    fn is_valid_placement(&self, proj: &ShapeProjection<D::Coordinate>, dim: &D) -> bool {
        if proj.len() != self.len() {
            return false;
        }
        let mut proj = proj.iter();
        // Self len must be nonzero and proj len must equal self len, so unwrap is safe.
        let start = proj.next().unwrap();
        let mut previous = start;
        for coord in proj {
            if dim.is_neighbor(coord, previous) && dim.is_colinear(start, previous, coord) {
                previous = coord;
            } else {
                return false;
            }
        }
        return true;
    }
}

/// State of the projection iterator for Line shape.
pub struct LineProjectIterState<C> {
    start: C,
    directions: Vec<C>,
    next_dir: usize,
}

/// State type for the ship projection iterator.
impl<D: Dimensions + ColinearCheck + ?Sized> ProjectIterState<D>
    for LineProjectIterState<D::Coordinate>
{
    type ShipShape = Line;

    /// Construct an instance of this iter state given the arguments.
    fn start(shape: &Self::ShipShape, dim: &D, coord: D::Coordinate) -> Self {
        if shape.len() == 1 {
            Self {
                start: coord,
                directions: Vec::new(),
                next_dir: 0,
            }
        } else {
            Self {
                start: coord.clone(),
                directions: dim.neighbors(coord).collect(),
                next_dir: 0,
            }
        }
    }

    /// Get the next possible projection of the ship's shape.
    fn next(&mut self, shape: &Self::ShipShape, dim: &D) -> Option<ShapeProjection<D::Coordinate>> {
        if shape.len() == 1 {
            if self.next_dir == 0 {
                self.next_dir = 1;
                Some(vec![self.start.clone()])
            } else {
                None
            }
        } else {
            loop {
                if self.next_dir < self.directions.len() {
                    let dir = self.directions[self.next_dir].clone();
                    self.next_dir += 1;
                    if let Some(route) = try_build_route(dim, shape.0, self.start.clone(), dir) {
                        return Some(route);
                    }
                } else {
                    return None;
                }
            }
        }
    }
}

/// Attempt to build a route in the given direction from the start.
fn try_build_route<D: Dimensions + ColinearCheck + ?Sized>(
    dim: &D,
    len: usize,
    start: D::Coordinate,
    dir: D::Coordinate,
) -> Option<ShapeProjection<D::Coordinate>> {
    let mut route = Vec::with_capacity(len);
    let mut visited = HashSet::with_capacity(len);
    route.push(start.clone());
    visited.insert(start.clone());
    route.push(dir.clone());
    visited.insert(dir.clone());
    let mut last = dir.clone();

    // Search out along the direction until the length is reached.
    'outer: while route.len() < len {
        // Check the neighbors of the last cell, to find one that's in the same direction
        // and not yet visited.
        for neighbor in dim.neighbors(last) {
            if dim.is_colinear(&start, &dir, &neighbor) && visited.insert(neighbor.clone()) {
                route.push(neighbor.clone());
                last = neighbor;
                // Once we find a neighbor at this position along the route, continue.
                continue 'outer;
            }
        }
        // If no neighbor matches along the line, there's nowhere else to go from here.
        return None;
    }
    return Some(route);
}
