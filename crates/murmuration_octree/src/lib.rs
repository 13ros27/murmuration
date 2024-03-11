use nonmax::NonMaxU32;
use slab::Slab;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

use point::PointData;

mod add;
mod get;
mod impls;
mod point;
mod remove;

pub use point::{ordered::OrderedBinary, Point};

pub struct Octree<D, P: Point> {
    branches: Slab<Branch<D, P>>,
    root: Option<BranchKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BranchKey(NonMaxU32);

enum Branch<D, P: Point> {
    Split {
        children: [Option<BranchKey>; 8],
        occupied: u8, // How many of the children are Some(_) (used for .remove).
        depth: u8,    // Equivalent to point_depth + 1 if there is a Skip above them
    },
    Skip {
        point: PointData<P>,
        point_depth: u8,
        child: BranchKey,
    },
    Leaf {
        point: PointData<P>,
        data: D,
        child: Option<BranchKey>,
    },
}

impl<D, P: Point> Branch<D, P> {
    fn new_data(point: PointData<P>, data: D) -> Self {
        Branch::Leaf {
            point,
            data,
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
        // SAFETY: It shouldn't be possible for the hierarchy to be incorrect
        // unsafe { self.branches.get_unchecked(key as usize) } // TODO: Change back
        self.branches.get(key as usize).unwrap()
    }

    fn get_branch_mut(&mut self, branch: BranchKey) -> &mut Branch<D, P> {
        let key: u32 = branch.0.into();
        // SAFETY: It shouldn't be possible for the hierarchy to be incorrect
        // unsafe { self.branches.get_unchecked_mut(key as usize) }
        self.branches.get_mut(key as usize).unwrap()
    }

    fn add_branch(&mut self, branch: Branch<D, P>) -> BranchKey {
        let key = self.branches.insert(branch);
        BranchKey(NonMaxU32::new(key.try_into().unwrap()).expect("Octree key overflowed 2^32-1"))
    }

    fn remove_branch(&mut self, branch: BranchKey) {
        let key: u32 = branch.0.into();
        self.branches.remove(key as usize);
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of branches currently in the tree (will typically be around 2 * items)
    pub fn num_branches(&self) -> usize {
        self.branches.len()
    }
}

impl<D: PartialEq, P: Point> Octree<D, P> {
    pub fn move_data(&mut self, old_point: &P, new_point: &P, data: D) -> bool {
        if let Ok((leaf, parents)) = self.get_leaf_parents(&old_point.get_point()) {
            if let Branch::Leaf {
                child,
                data: leaf_data,
                ..
            } = self.get_branch(leaf)
            {
                // Optimise trivial moves
                if &data == leaf_data && child.is_none() {
                    let trivial = if parents.is_empty() {
                        true
                    } else {
                        let mut depth = 0;
                        for parent in &parents {
                            match self.get_branch(**parent) {
                                Branch::Split { .. } => depth += 1,
                                Branch::Skip { point_depth, .. } => {
                                    depth += point_depth;
                                    break;
                                }
                                Branch::Leaf { .. } => unreachable!(),
                            }
                        }
                        let shared =
                            (&old_point.get_point() ^ &new_point.get_point()).leading_zeros();
                        shared >= depth
                    };

                    if trivial {
                        let Branch::Leaf { point, .. } = self.get_branch_mut(leaf) else {
                            unreachable!()
                        };
                        *point = new_point.get_point();
                        return true;
                    }
                }
            }

            if self.remove_from_parent_chain(leaf, parents, &data) {
                self.add(new_point, data);
                return true;
            }
        }
        false
    }
}

// Manual impl to add the P::Data: Debug bound
impl<D, P> Debug for Branch<D, P>
where
    D: Debug,
    P: Point + Debug,
    P::Data: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Branch::Split {
                children,
                occupied,
                depth,
            } => f
                .debug_struct("Branch::Split")
                .field("children", children)
                .field("occupied", occupied)
                .field("depth", depth)
                .finish(),
            Branch::Skip {
                point,
                point_depth,
                child,
            } => f
                .debug_struct("Branch::Skip")
                .field("point", point)
                .field("point_depth", point_depth)
                .field("child", child)
                .finish(),
            Branch::Leaf { point, data, child } => f
                .debug_struct("Branch::Leaf")
                .field("point", point)
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
