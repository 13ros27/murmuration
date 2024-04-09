use std::collections::VecDeque;
use std::iter::FusedIterator;

use super::{
    point::{ordered::OrderedBinary, Point, PointData},
    Branch, BranchKey, Octree,
};

struct Within<'a, D, P: Point> {
    octree: &'a Octree<D, P>,
    centre: PointData<P>,
    sqr_dist: P::Data,
    parents: VecDeque<(BranchKey, Option<u8>)>,
    leaf: Option<BranchKey>,
    point: PointData<P>, // The last point value we got from a Skip parent (plus info from Split)
}

impl<'a, D, P: Point> Iterator for Within<'a, D, P> {
    type Item = &'a D;
    fn next(&mut self) -> Option<&'a D> {
        if let Some(leaf) = self.leaf {
            if let Branch::Leaf {
                child: Some(child), ..
            } = self.octree.get_branch(leaf)
            {
                self.leaf = Some(*child);
                let Branch::Leaf { data, .. } = self.octree.get_branch(*child) else {
                    unreachable!()
                };
                return Some(data);
            }
            self.leaf = None;
        }

        let mut moving_up = false;
        'outer: loop {
            if self.parents.is_empty() {
                return None;
            }
            match self.octree.get_branch(self.parents[0].0) {
                Branch::Split {
                    children, depth, ..
                } => {
                    for i in self.parents[0].1.map_or(0, |n| n + 1)..8 {
                        if let Some(child) = children[i as usize] {
                            let closest = self.point.closest_distance(i, &self.centre, *depth);
                            if closest <= self.sqr_dist || closest.is_irrelevant() {
                                self.parents[0].1 = Some(i);
                                self.parents.push_front((child, None));
                                moving_up = false;
                                if depth != &P::MAX_DEPTH {
                                    self.point = self.point.combine_ind(i, *depth);
                                }
                                continue 'outer;
                            }
                        }
                    }
                    // There are no more valid branches in here
                    moving_up = true;
                    self.parents.pop_front();
                }
                Branch::Leaf { data, point, .. } => {
                    if point.distance_squared(&self.centre) <= self.sqr_dist {
                        self.leaf = Some(self.parents[0].0);
                        self.parents.pop_front();
                        return Some(data);
                    }
                    self.parents.pop_front();
                }
                Branch::Skip { point, child, .. } => {
                    if moving_up {
                        self.parents.pop_front();
                    } else {
                        self.point = point.clone();
                        self.parents.push_front((*child, None));
                    }
                }
            }
        }
    }
}

impl<D, P: Point> FusedIterator for Within<'_, D, P> {}

impl<D, P: Point> Octree<D, P> {
    pub fn within(&self, point: &P, distance: P::Data) -> impl Iterator<Item = &D> {
        let root = self
            .root
            .into_iter()
            .map(|v| (v, None))
            .collect::<VecDeque<_>>();
        Within {
            octree: self,
            centre: point.get_point(),
            sqr_dist: distance.clone() * distance,
            parents: root,
            leaf: None,
            point: PointData::<P>::ZERO,
        }
    }
}
