use std::collections::VecDeque;
use std::ops::Deref;

use super::{
    point::{Point, PointData},
    Branch, BranchKey, Octree,
};

impl<D: PartialEq, P: Point> Octree<D, P> {
    /// Removes the given `data` from `point` in the tree if it exists, otherwise returns `false`.
    pub fn remove(&mut self, point: &P, data: &D) -> bool {
        let point = point.get_point();
        if let Ok((leaf, parents)) = self.get_leaf_parents(&point) {
            self.remove_from_parent_chain(leaf, parents, data)
        } else {
            false
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn remove_from_parent_chain(
        &mut self,
        leaf: BranchKey,
        parents: VecDeque<ParentBranch>,
        data: &D,
    ) -> bool {
        let mut leaf = leaf;
        let mut parents = parents;
        let child = loop {
            let Branch::Leaf {
                data: leaf_data,
                child,
                ..
            } = self.get_branch(leaf)
            else {
                unreachable!()
            };

            if data == leaf_data {
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
                let info = match self.get_branch_mut(**parent) {
                    Branch::Leaf { child, .. } => {
                        *child = None;
                        None
                    }
                    Branch::Split {
                        children,
                        occupied,
                        depth,
                    } => {
                        *occupied -= 1;
                        let ind = parent.ind.unwrap() as usize;
                        if *occupied > 1 {
                            children[ind] = None;
                            None
                        } else {
                            Some((
                                children
                                    .iter()
                                    .enumerate()
                                    .find_map(|(i, b)| match b {
                                        Some(b) if i != ind => Some(*b),
                                        _ => None,
                                    })
                                    .unwrap(),
                                *depth,
                            ))
                        }
                    }
                    Branch::Skip { .. } => unreachable!(),
                };

                // If there is a new_child we want to re-parent it onto the item above
                if let Some((child, depth)) = info {
                    let mut have_split = false;
                    let mut sub_child = child;
                    let child_point = loop {
                        match self.get_branch(sub_child) {
                            Branch::Leaf { point, .. } | Branch::Skip { point, .. } => break point,
                            Branch::Split { children, .. } => {
                                if let Some(child) = children.iter().flatten().next() {
                                    sub_child = *child;
                                    have_split = true;
                                }
                            }
                        }
                    };
                    let skip_branch = if have_split {
                        self.add_branch(Branch::Skip {
                            point: child_point.clone(),
                            point_depth: depth,
                            child,
                        })
                    } else {
                        child
                    };

                    self.remove_branch(**parent);
                    let mut reparented = false;
                    for parent in parents.iter().skip(1) {
                        if let Branch::Split { children, .. } = self.get_branch_mut(**parent) {
                            children[parent.ind.unwrap() as usize] = Some(skip_branch);
                            reparented = true;
                            break;
                        }
                        self.remove_branch(**parent);
                    }

                    if !reparented {
                        self.root = Some(skip_branch);
                    }
                }
            }
        } else {
            self.root = *child;
        }
        self.remove_branch(leaf);
        true
    }
}

impl<D, P: Point> Octree<D, P> {
    /// Returns the leaf and chain of branches leading to it
    pub(crate) fn get_leaf_parents(
        &self,
        point: &PointData<P>,
    ) -> Result<(BranchKey, VecDeque<ParentBranch>), VecDeque<ParentBranch>> {
        let Some(mut branch) = self.root else {
            return Err(VecDeque::new());
        };
        let mut parents = VecDeque::new();
        let mut depth = 0;

        loop {
            match self.get_branch(branch) {
                Branch::Leaf {
                    point: skip_point, ..
                } => {
                    return (point == skip_point)
                        .then_some((branch, parents.clone()))
                        .ok_or(parents);
                }
                Branch::Skip {
                    point: skip_point,
                    point_depth: skip_depth,
                    child,
                } => {
                    let shared = (point ^ skip_point).leading_zeros();
                    if shared >= *skip_depth {
                        parents.push_front(ParentBranch { branch, ind: None });
                        branch = *child;
                        depth = *skip_depth;
                    } else {
                        return Err(parents);
                    }
                }
                Branch::Split { children, .. } => {
                    let ind = point.nth(depth);
                    if let Some(child) = children[ind as usize] {
                        parents.push_front(ParentBranch {
                            branch,
                            ind: Some(ind),
                        });
                        branch = child;
                        depth += 1;
                    } else {
                        return Err(parents);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ParentBranch {
    pub(crate) branch: BranchKey,
    pub(crate) ind: Option<u8>,
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
                children[self.ind.unwrap() as usize] = Some(new_child);
            }
        }
    }
}
