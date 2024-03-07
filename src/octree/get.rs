use std::collections::VecDeque;
use std::iter::FusedIterator;

use super::{
    point::{Point, PointData},
    remove::ParentBranch,
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
    leaf: Option<BranchKey>,
    parents: VecDeque<ParentBranch>,
    handled_branches: VecDeque<Vec<u8>>, // TODO: Maybe a vector of bitsets
}

impl<D, P: Point> Within<'_, D, P> {
    fn is_possible_child(&self, point: &PointData<P>) -> bool {
        let mut depth = 0;
        for parent in &self.parents {
            match self.octree.get_branch(**parent) {
                Branch::Split { .. } => depth += 1,
                Branch::Skip { point_depth, .. } => {
                    depth += point_depth;
                    break;
                }
                Branch::Leaf { .. } => unreachable!(),
            }
        }

        let closest = point.approximate_closest(&self.centre, depth - 1);
        closest.distance_squared(point) <= self.sqr_dist
    }

    fn propagate_down(&mut self, mut branch: BranchKey) -> bool {
        loop {
            match self.octree.get_branch(branch) {
                Branch::Leaf { point, .. } => {
                    if point.distance_squared(&self.centre) <= self.sqr_dist {
                        self.leaf = Some(branch);
                        return true;
                    } else {
                        return false;
                    }
                }
                Branch::Skip {
                    child: new_child, ..
                } => {
                    self.parents.push_front(ParentBranch { branch, ind: None });
                    branch = *new_child;
                }
                Branch::Split { children, .. } => {
                    let mut first = None;
                    for i in 0..8 {
                        if let Some(child) = children[i as usize] {
                            self.handled_branches.push_front(vec![i]);
                            first = Some((child, i));
                            break;
                        }
                    }
                    let first = first.unwrap();
                    self.parents.push_front(ParentBranch {
                        branch,
                        ind: Some(first.1),
                    });
                    branch = first.0;
                }
            }
        }
    }
}

impl<'a, D, P: Point> Iterator for Within<'a, D, P> {
    type Item = &'a D;
    fn next(&mut self) -> Option<&'a D> {
        let Some(leaf) = self.leaf else {
            return None;
        };
        let Branch::Leaf { data, child, point } = self.octree.get_branch(leaf) else {
            unreachable!()
        };

        if let Some(child) = child {
            self.leaf = Some(*child);
            Some(data)
        } else if !self.parents.is_empty() {
            loop {
                let child = 'outer: loop {
                    let parent = &self.parents[0];
                    match self.octree.get_branch(**parent) {
                        Branch::Skip { .. } => {}
                        Branch::Split { children, occupied } => {
                            // TODO: Probably should do possible child check on the branches rather than the Skip root
                            let is_possible_child = self.is_possible_child(point);
                            if let Some(handled) = self.handled_branches.front_mut() {
                                if handled.len() < *occupied as usize && is_possible_child {
                                    for i in (*handled.last().unwrap() + 1)..8 {
                                        if i != handled[0] {
                                            if let Some(child) = children[i as usize] {
                                                handled.push(i);
                                                break 'outer Some(child);
                                            }
                                        }
                                    }
                                }
                                // If we don't find a possible child we should remove this from
                                //  handled branch because we are moving up to the parent above
                                self.handled_branches.pop_front();
                            } else if is_possible_child {
                                // We are moving up into a new Split branch so we shouldn't go back
                                //  down the branch we came up from
                                let handled = parent.ind.unwrap();
                                for i in 0..8 {
                                    if i != handled {
                                        if let Some(child) = children[i as usize] {
                                            self.handled_branches.push_front(vec![handled, i]);
                                            break 'outer Some(child);
                                        }
                                    }
                                }
                            }
                        }
                        Branch::Leaf { .. } => unreachable!(),
                    }

                    self.parents.pop_front();
                    if self.parents.is_empty() {
                        break None;
                    }
                };

                // We have reached a child branch we haven't checked yet (that could be valid)
                if let Some(child) = child {
                    if self.propagate_down(child) {
                        return Some(data);
                    }
                } else {
                    return Some(data);
                }
            }
        } else {
            None
        }
    }
}

impl<D, P: Point> Octree<D, P> {
    pub fn get(&self, point: P) -> impl Iterator<Item = &D> {
        let leaf = self.get_leaf(point.get_point());
        GetIter { octree: self, leaf }
    }

    pub fn get_single(&self, point: P) -> Option<&D> {
        self.get(point).next()
    }

    pub fn within(&self, point: P, distance: P::Data) -> impl Iterator<Item = &D> {
        match self.get_leaf_parents(point.get_point()) {
            Ok((leaf, parents)) => Within {
                octree: self,
                centre: point.get_point(),
                sqr_dist: distance.clone() * distance,
                leaf: Some(leaf),
                parents,
                handled_branches: VecDeque::new(),
            },
            Err(parents) => {
                let first_parent = *parents[0];
                // TODO: This actually does need to be processed properly (we can do Within not around a point)
                let mut this = Within {
                    octree: self,
                    centre: point.get_point(),
                    sqr_dist: distance.clone() * distance,
                    leaf: None,
                    parents,
                    handled_branches: VecDeque::new(),
                };
                this.propagate_down(first_parent);
                this
            }
        }
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
