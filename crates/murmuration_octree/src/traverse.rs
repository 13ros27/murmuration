use crate::{Branch, BranchKey, Octree, OrderedBinary, Point, PointData};
use std::collections::VecDeque;
use std::iter::FusedIterator;
use std::marker::PhantomData;

trait Visitor<'a, D, P: Point> {
    fn visit_split(
        &mut self,
        cursor: &mut Cursor<D, P>,
        children: &'a [Option<BranchKey>; 8],
    ) -> Option<&'a D>;
    fn visit_skip(&mut self, cursor: &mut Cursor<D, P>) -> Option<&'a D>;
    fn visit_leaf(&mut self, cursor: &mut Cursor<D, P>, data: &'a D) -> Option<&'a D>;
}

struct Within<P: Point> {
    centre: PointData<P>,
    sqr_dist: P::Data,
}

impl<'a, D, P: Point> Visitor<'a, D, P> for Within<P>
where
    <P as Point>::Data: std::fmt::Debug,
{
    fn visit_split(
        &mut self,
        cursor: &mut Cursor<D, P>,
        _children: &'a [Option<BranchKey>; 8],
    ) -> Option<&'a D> {
        // println!("{:?}", cursor.children().collect::<Vec<_>>());
        for ind in cursor.children() {
            // println!("Before next");
            cursor.next(ind);
            // println!("Post next");
            let closest = cursor
                .point
                .closest_distance_new(&self.centre, cursor.depth);
            println!(
                "Closest {:?}, {:?}, {:?}, {:?}",
                closest, cursor.point, self.centre, cursor.depth
            );
            // println!("{:?}", closest);
            // println!(
            //     "Distance: {:?}->{:?} is {:?} at depth {:?}, {:?}",
            //     cursor.point,
            // [
            //     P::Data::from_ordered(cursor.point.0[0]),
            //     P::Data::from_ordered(cursor.point.0[1]),
            //     P::Data::from_ordered(cursor.point.0[2])
            // ],
            //     self.centre,
            //     cursor.point.distance_squared(&self.centre),
            //     cursor.depth,
            //     cursor
            //         .point
            //         .closest_distance_new(&self.centre, cursor.depth)
            // );
            if closest <= self.sqr_dist || closest.is_irrelevant() {
                return None;
            }
            cursor.prev();
        }
        // println!("Hello");
        None
    }

    fn visit_skip(&mut self, cursor: &mut Cursor<D, P>) -> Option<&'a D> {
        cursor.next(0);
        None
    }

    fn visit_leaf(&mut self, cursor: &mut Cursor<D, P>, data: &'a D) -> Option<&'a D> {
        // println!(
        //     "Distance: {:?}->{:?} is {:?}",
        //     [
        //         P::Data::from_ordered(cursor.point.0[0]),
        //         P::Data::from_ordered(cursor.point.0[1]),
        //         P::Data::from_ordered(cursor.point.0[2])
        //     ],
        //     self.centre,
        //     cursor.point.distance_squared(&self.centre)
        // );
        if cursor.point.distance_squared(&self.centre) <= self.sqr_dist {
            if cursor.next(0).is_none() {
                cursor.prev();
            }
            Some(data)
        } else {
            cursor.prev();
            None
        }
    }
}

struct CursorChildren {
    tested: u8, // I would prefer to actually change the cursor in this but can't work it out
}

impl Iterator for CursorChildren {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let child = self.tested.trailing_ones();
        if child != 8 {
            self.tested |= 1_u8.wrapping_shl(child);
            Some(child as u8)
        } else {
            None
        }
    }
}

struct Cursor<'a, D, P: Point> {
    tree: &'a Octree<D, P>,
    point: PointData<P>,
    depth: u8,
    parents: VecDeque<(BranchKey, u8)>, // The u8 is a bitflag of tested paths (or 0 if irrelevant)
}

impl<'a, D, P: Point> Cursor<'a, D, P>
// where
//     <P as Point>::Data: std::fmt::Debug,
{
    fn root(tree: &'a Octree<D, P>) -> Self {
        let mut parents = VecDeque::new();
        if let Some(branch) = tree.root {
            if let Branch::Split { occupied, .. } = tree.get_branch(branch) {
                parents.push_front((branch, !occupied));
            } else {
                parents.push_front((branch, 0));
            }
        }

        Self {
            tree,
            point: PointData::ZERO,
            depth: 0,
            parents,
        }
    }

    fn get(&self) -> Option<&'a Branch<D, P>> {
        Some(self.tree.get_branch(self.parents.front()?.0))
    }

    fn next(&mut self, child: u8) -> Option<BranchKey> {
        let branch = match self.get() {
            Some(Branch::Split {
                children, depth, ..
            }) => {
                self.depth = *depth;
                if let Some(branch) = children[child as usize] {
                    // Mark this child (`branch`) as tested in the `parents` queue.
                    self.parents[0].1 |= 1_u8.wrapping_shl(child as u32);
                    // Combine this child index with the current point to form the child point.
                    self.point = self.point.combine_ind(child, *depth);
                    Some(branch)
                } else {
                    None
                }
            }
            Some(Branch::Skip {
                child,
                point,
                point_depth,
                ..
            }) => {
                self.depth = *point_depth;
                self.point = point.clone();
                Some(*child)
            }
            Some(Branch::Leaf { child, .. }) => {
                self.depth = 32;
                *child
            }
            None => None,
        };

        if let Some(branch) = branch {
            let tested = match self.tree.get_branch(branch) {
                Branch::Split {
                    occupied, depth, ..
                } => {
                    // If we are stepping into a new `Branch::Split` then we haven't tested any of
                    // the occupied branches.
                    !occupied
                }
                Branch::Skip { point, .. } => {
                    self.point = point.clone();
                    0
                }
                Branch::Leaf { point, .. } => {
                    self.point = point.clone();
                    0
                }
            };
            self.parents.push_front((branch, tested));
        }
        branch
    }

    // Returns whether there was a previous element
    fn prev(&mut self) -> bool {
        loop {
            self.parents.pop_front();
            if let Some((branch, tested)) = self.parents.front() {
                match (self.tree.get_branch(*branch), tested) {
                    (Branch::Split { .. }, 255)
                    | (Branch::Skip { .. } | Branch::Leaf { .. }, _) => {} // Skip through any processed branches (which includes fully tested `Branch::Split`)
                    (Branch::Split { depth, .. }, _) => {
                        self.depth = *depth;
                        return true;
                    }
                }
            } else {
                self.depth = 0;
                return false;
            }
        }
    }

    fn children(&self) -> impl Iterator<Item = u8> {
        CursorChildren {
            tested: self.parents.front().unwrap().1,
        }
    }

    // Returns the next untested child
    fn next_child(&mut self) -> u8 {
        if let Some((_, tested)) = self.parents.front() {
            // println!("Tested {:?}", self.parents);
            tested.trailing_ones() as u8
        } else {
            0
        }
    }

    fn tried_all_children(&self) -> bool {
        if let Some((_, tested)) = self.parents.front() {
            *tested == 255
        } else {
            true
        }
    }
}

struct Traverser<'a, D, P: Point, V: Visitor<'a, D, P>> {
    visitor: V,
    cursor: Cursor<'a, D, P>,
    _phantom: PhantomData<(&'a (), D, P)>,
}

impl<'a, D: 'a, P: Point, V: Visitor<'a, D, P>> Iterator for Traverser<'a, D, P, V>
// where
//     <P as Point>::Data: std::fmt::Debug,
{
    type Item = &'a D;
    fn next(&mut self) -> Option<&'a D> {
        loop {
            let yielded = match self.cursor.get()? {
                Branch::Split { children, .. } => {
                    self.visitor.visit_split(&mut self.cursor, &children)
                }
                Branch::Skip { .. } => self.visitor.visit_skip(&mut self.cursor),
                Branch::Leaf { data, .. } => self.visitor.visit_leaf(&mut self.cursor, data),
            };
            if let Some(value) = yielded {
                return Some(value);
            }
        }
    }
}

impl<'a, D: 'a, P: Point, V: Visitor<'a, D, P>> FusedIterator for Traverser<'a, D, P, V>
// where
// <P as Point>::Data: std::fmt::Debug
{
}

impl<D, P: Point> Octree<D, P>
where
    <P as Point>::Data: std::fmt::Debug,
{
    fn traverse<'a, V: Visitor<'a, D, P>>(&'a self, visitor: V) -> impl Iterator<Item = &D> {
        Traverser {
            visitor,
            cursor: Cursor::root(self),
            _phantom: PhantomData,
        }
    }

    /// TODO: Docs
    pub fn within_new(&self, point: &P, distance: P::Data) -> impl Iterator<Item = &D> {
        self.traverse(Within {
            centre: point.get_point(),
            sqr_dist: distance.clone() * distance,
        })
    }
}
