use super::{point::Point, Branch, BranchKey, Octree};
use crate::octree::point::PointData;
use std::iter::FusedIterator;

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
impl<D, P: Point> Octree<D, P> {
    pub fn get(&self, point: P) -> impl Iterator<Item = &D> {
        let leaf = self.get_leaf(point.get_point());
        GetIter { octree: self, leaf }
    }

    pub fn get_single(&self, point: P) -> Option<&D> {
        self.get(point).next()
    }

    fn get_leaf(&self, point: PointData<P>) -> Option<BranchKey> {
        let Some(mut branch) = self.root else {
            return None;
        };
        let mut depth = 0;

        loop {
            match self.get_branch(branch) {
                Branch::Leaf {
                    point: skip_point, ..
                } => {
                    return (&point == skip_point).then_some(branch);
                }
                Branch::Skip {
                    point: skip_point,
                    point_depth: skip_depth,
                    child,
                } => {
                    let shared = (&point ^ skip_point).leading_zeros();
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
