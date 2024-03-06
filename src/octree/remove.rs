use std::collections::VecDeque;
use std::ops::Deref;

use super::{
    point::{Point, PointData},
    Branch, BranchKey, Octree,
};

impl<D: PartialEq, P: Point> Octree<D, P> {
    pub fn remove(&mut self, point: P, data: D) -> bool {
        let point = point.get_point();
        if let Some((mut leaf, mut parents)) = self.get_leaf_parents(point) {
            let child = loop {
                let Branch::Leaf {
                    data: leaf_data,
                    child,
                    ..
                } = self.get_branch(leaf)
                else {
                    unreachable!()
                };

                if &data == leaf_data {
                    break child;
                } else if let Some(child) = child {
                    parents.push_front(ParentBranch {
                        branch: leaf,
                        ind: None,
                    });
                    leaf = *child;
                } else {
                    return false;
                }
            };

            if let Some(parent) = parents.front() {
                if let Some(new_child) = child {
                    parent.set_child(self, *new_child);
                } else {
                    let new_child = match self.get_branch_mut(**parent) {
                        Branch::Leaf { child, .. } => {
                            *child = None;
                            None
                        }
                        Branch::Split { children, occupied } => {
                            *occupied -= 1;
                            let ind = parent.ind.unwrap() as usize;
                            if *occupied > 1 {
                                children[ind] = None;
                                None
                            } else {
                                Some(
                                    children
                                        .iter()
                                        .enumerate()
                                        .find_map(|(i, b)| match b {
                                            Some(b) if i != ind => Some(*b),
                                            _ => None,
                                        })
                                        .unwrap(),
                                )
                            }
                        }
                        Branch::Skip { .. } => unreachable!(),
                    };

                    // If there is a new_child we want to re-parent it onto the item above
                    if let Some(child) = new_child {
                        self.remove_branch(**parent);
                        let mut reparented = false;
                        for parent in parents.iter().skip(1) {
                            if let Branch::Split { children, .. } = self.get_branch_mut(**parent) {
                                children[parent.ind.unwrap() as usize] = Some(child);
                                reparented = true;
                                break;
                            }
                            self.remove_branch(**parent);
                        }

                        if !reparented {
                            self.root = Some(child);
                        }
                    }
                }
            } else {
                self.root = *child;
            }
            self.remove_branch(leaf);
            true
        } else {
            false
        }
    }
}

impl<D, P: Point> Octree<D, P> {
    /// Returns the leaf and chain of branches leading to it
    fn get_leaf_parents(&self, point: PointData<P>) -> Option<(BranchKey, VecDeque<ParentBranch>)> {
        let Some(mut branch) = self.root else {
            return None;
        };
        let mut parents = VecDeque::new();
        let mut depth = 0;

        loop {
            match self.get_branch(branch) {
                Branch::Leaf {
                    point: skip_point, ..
                } => {
                    return (&point == skip_point).then_some((branch, parents));
                }
                Branch::Skip {
                    point: skip_point,
                    point_depth: skip_depth,
                    child,
                } => {
                    let shared = (&point ^ skip_point).leading_zeros();
                    if shared >= *skip_depth {
                        parents.push_front(ParentBranch { branch, ind: None });
                        branch = *child;
                        depth = *skip_depth;
                    } else {
                        return None;
                    }
                }
                Branch::Split { children, .. } => {
                    let ind = point.nth(depth) as usize;
                    if let Some(child) = children[ind] {
                        parents.push_front(ParentBranch {
                            branch,
                            ind: Some(ind as u8),
                        });
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

struct ParentBranch {
    branch: BranchKey,
    ind: Option<u8>,
}

impl Deref for ParentBranch {
    type Target = BranchKey;
    fn deref(&self) -> &Self::Target {
        &self.branch
    }
}

impl ParentBranch {
    fn set_child<D, P: Point>(&self, octree: &mut Octree<D, P>, new_child: BranchKey) {
        match octree.get_branch_mut(self.branch) {
            Branch::Leaf { child, .. } => *child = Some(new_child),
            Branch::Skip { child, .. } => *child = new_child,
            Branch::Split { children, .. } => {
                children[self.ind.unwrap() as usize] = Some(new_child)
            }
        }
    }
}
