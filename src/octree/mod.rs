use nonmax::NonMaxU32;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

use slab::Slab;

use point::{Point, PointData};

pub mod point;

pub struct Octree<D, P: Point> {
    branches: Slab<Branch<D, P>>,
    root: Option<BranchKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BranchKey(NonMaxU32);

enum Branch<D, P: Point> {
    Split([Option<BranchKey>; 8]),
    Skip {
        point: PointData<P>,
        point_depth: u8,
        data: Option<D>,
        child: Option<BranchKey>,
    },
}

impl<D, P: Point> Branch<D, P> {
    fn new_data(point: PointData<P>, data: D) -> Self {
        Branch::Skip {
            point,
            point_depth: P::MAX_DEPTH,
            data: Some(data),
            child: None,
        }
    }
}

impl<D, P: Point> Default for Octree<D, P> {
    fn default() -> Self {
        Self {
            branches: Slab::new(),
            root: None,
        }
    }
}

impl<D, P: Point> Octree<D, P> {
    fn get_branch(&self, branch: BranchKey) -> &Branch<D, P> {
        let key: u32 = branch.0.into();
        self.branches.get(key as usize).unwrap()
    }

    fn get_branch_mut(&mut self, branch: BranchKey) -> &mut Branch<D, P> {
        let key: u32 = branch.0.into();
        self.branches.get_mut(key as usize).unwrap()
    }

    fn add_branch(&mut self, branch: Branch<D, P>) -> BranchKey {
        let key = self.branches.insert(branch);
        BranchKey(NonMaxU32::new(key as u32).expect("Octree key overflowed 2^32-1"))
    }

    fn remove_branch(&mut self, branch: BranchKey) {
        let key: u32 = branch.0.into();
        self.branches.remove(key as usize);
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_single(&self, point: P) -> Option<&D> {
        let point = point.get_point();
        self.root
            .and_then(|b| self.get_single_from_branch(b, point, 0))
    }

    fn get_single_from_branch(
        &self,
        branch: BranchKey,
        point: PointData<P>,
        depth: u8,
    ) -> Option<&D> {
        match self.get_branch(branch) {
            Branch::Skip {
                point: skip_point,
                point_depth: skip_depth,
                data,
                child,
            } => {
                let shared = (&point ^ skip_point).leading_zeros();
                if shared == 32 {
                    data.as_ref()
                } else {
                    if shared < *skip_depth {
                        None
                    } else if let Some(child) = child {
                        self.get_single_from_branch(*child, point, *skip_depth)
                    } else {
                        None
                    }
                }
            }
            Branch::Split(children) => {
                let ind = point.nth(depth) as usize;
                if let Some(child) = children[ind] {
                    self.get_single_from_branch(child, point, depth + 1)
                } else {
                    None
                }
            }
        }
    }

    pub fn add(&mut self, point: P, data: D) {
        let point = point.get_point();
        if let Some(child_key) = self.root {
            if let Some(branch) = self.add_to_branch(child_key, data, point, 0) {
                self.root = Some(branch);
            }
        } else {
            let branch = self.add_branch(Branch::Skip {
                point,
                point_depth: P::MAX_DEPTH,
                data: Some(data),
                child: None,
            });
            self.root = Some(branch);
        }
    }
}

impl<D, P: Point> Octree<D, P> {
    // NB: The returned Option<BranchKey> is to change the branch above in the recursive chain
    fn add_to_branch(
        &mut self,
        branch: BranchKey,
        data: D,
        point: PointData<P>,
        depth: u8,
    ) -> Option<BranchKey> {
        match self.get_branch(branch) {
            Branch::Skip {
                point: child_point,
                point_depth,
                child: branch_child,
                ..
            } => {
                let shared = (&point ^ child_point).leading_zeros();
                if shared >= *point_depth {
                    // They share all their data (up to point depth)
                    if let Some(branch_key) = branch_child {
                        if let Some(new) =
                            self.add_to_branch(*branch_key, data, point, *point_depth)
                        {
                            self.set_skip_child(branch, new);
                        }
                    } else {
                        let new = self.add_branch(Branch::new_data(point, data));
                        self.add_duplicate(branch, new);
                    }
                    None
                } else {
                    let child_point = child_point.clone();
                    let new = self.add_branch(Branch::new_data(point.clone(), data));
                    let split = self.add_new_split(new, branch, &point, &child_point, shared);

                    if shared > 0 {
                        let new = self.add_branch(Branch::Skip {
                            point,
                            point_depth: shared,
                            data: None,
                            child: Some(split),
                        });
                        Some(new)
                    } else {
                        Some(split)
                    }
                }
            }
            Branch::Split(children) => {
                let ind = point.nth(depth) as usize;
                if let Some(child) = children[ind] {
                    if let Some(new) = self.add_to_branch(child, data, point, depth + 1) {
                        self.set_split_child(branch, ind, new);
                    }
                } else {
                    let new = self.add_branch(Branch::new_data(point, data));
                    self.set_split_child(branch, ind, new);
                }
                None
            }
        }
    }

    /// Add a new split item between child1@point1 and child2@point2 (splitting at depth)
    fn add_new_split(
        &mut self,
        child1: BranchKey,
        child2: BranchKey,
        point1: &PointData<P>,
        point2: &PointData<P>,
        depth: u8,
    ) -> BranchKey {
        let dir1 = point1.nth(depth) as usize;
        let dir2 = point2.nth(depth) as usize;
        let mut children = [None, None, None, None, None, None, None, None];
        children[dir1] = Some(child1);
        children[dir2] = Some(child2);
        self.add_branch(Branch::Split(children))
    }

    /// Add a duplicate point split to a chain
    fn add_duplicate(&mut self, dupe_branch: BranchKey, new_branch: BranchKey) {
        let Branch::Skip { child, .. } = self.get_branch_mut(dupe_branch) else {
            unreachable!()
        };
        *child = Some(new_branch);
    }

    /// Sets the child of the given branch to 'new' if it is a skip branch (N.B. must be passed a skip branch)
    fn set_skip_child(&mut self, branch: BranchKey, new: BranchKey) {
        let Branch::Skip { child, .. } = self.get_branch_mut(branch) else {
            unreachable!()
        };
        *child = Some(new);
    }

    /// Sets a child of the given branch to 'new' if it is a split branch (N.B. must be passed a split branch)
    fn set_split_child(&mut self, branch: BranchKey, ind: usize, new: BranchKey) {
        let Branch::Split(children) = self.get_branch_mut(branch) else {
            unreachable!()
        };
        children[ind] = Some(new);
    }
}

impl<D, P> Debug for Branch<D, P>
where
    D: Debug,
    P: Point + Debug,
    P::Data: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Branch::Split(c) => f.debug_tuple("Branch::Split").field(c).finish(),
            Branch::Skip {
                point,
                point_depth,
                data,
                child,
            } => f
                .debug_struct("Branch::Skip")
                .field("point", point)
                .field("point_depth", point_depth)
                .field("data", data)
                .field("child", child)
                .finish(),
        }
    }
}

impl<D, P> Debug for Octree<D, P>
where
    D: Debug,
    P: Point + Debug,
    P::Data: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Octree")
            .field("root", &self.root)
            .field(
                "branches",
                &self.branches.iter().collect::<BTreeMap<_, _>>(),
            )
            .finish()
    }
}
