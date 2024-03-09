use std::collections::VecDeque;
use std::iter::FusedIterator;

use super::{
    point::{OrderedBinary, Point, PointData},
    Branch, BranchKey, Octree,
};

struct GetIter<'a, D, P: Point> {
    octree: &'a Octree<D, P>,
    leaf: Option<BranchKey>,
}

impl<'a, D, P: Point> Iterator for GetIter<'a, D, P> {
    type Item = &'a D;
    fn next(&mut self) -> Option<&'a D> {
        if let Some(key) = self.leaf {
            if let Branch::Leaf { data, child, .. } = self.octree.get_branch(key) {
                self.leaf = *child;
                return Some(data);
            }
        }
        None
    }
}

impl<D, P: Point> FusedIterator for GetIter<'_, D, P> {}

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
    pub fn get(&self, point: &P) -> impl Iterator<Item = &D> {
        let leaf = self.get_leaf(&point.get_point());
        GetIter { octree: self, leaf }
    }

    pub fn get_single(&self, point: &P) -> Option<&D> {
        self.get(point).next()
    }

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

    fn get_leaf(&self, point: &PointData<P>) -> Option<BranchKey> {
        let Some(mut branch) = self.root else {
            return None;
        };
        let mut depth = 0;

        loop {
            match self.get_branch(branch) {
                Branch::Leaf {
                    point: skip_point, ..
                } => {
                    return (point == skip_point).then_some(branch);
                }
                Branch::Skip {
                    point: skip_point,
                    point_depth: skip_depth,
                    child,
                } => {
                    let shared = (point ^ skip_point).leading_zeros();
                    if shared >= *skip_depth {
                        branch = *child;
                        depth = *skip_depth;
                    } else {
                        return None;
                    }
                }
                Branch::Split { children, .. } => {
                    let ind = point.nth(depth) as usize;
                    if let Some(child) = children[ind] {
                        branch = child;
                        depth += 1;
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}
