use super::{
    point::{Point, PointData},
    Branch, BranchKey, Octree,
};

impl<D, P: Point> Octree<D, P> {
    /// Adds an item to the tree at the given point with `data`.
    pub fn add(&mut self, point: &P, data: D) {
        self.add_int(point.get_point(), data);
    }

    pub(crate) fn add_int(&mut self, point: PointData<P>, data: D) {
        if let Some(child_key) = self.root {
            if let Some(branch) = self.add_to_branch(child_key, data, point, 0) {
                self.root = Some(branch);
            }
        } else {
            let branch = self.add_branch(Branch::new_data(point, data));
            self.root = Some(branch);
        }
    }

    // NB: The returned Option<BranchKey> is to change the branch above in the recursive chain
    fn add_to_branch(
        &mut self,
        branch: BranchKey,
        data: D,
        point: PointData<P>,
        depth: u8,
    ) -> Option<BranchKey> {
        match self.get_branch(branch) {
            Branch::Leaf {
                point: child_point, ..
            } => {
                if &point == child_point {
                    Some(self.add_branch(Branch::Leaf {
                        point,
                        data,
                        child: Some(branch),
                    }))
                } else {
                    let shared = (&point ^ child_point).leading_zeros();
                    let child_point = child_point.clone();
                    let new = self.add_branch(Branch::new_data(point.clone(), data));
                    Some(self.add_new_split(new, branch, point, &child_point, shared, depth))
                }
            }
            Branch::Skip {
                point: child_point,
                point_depth,
                child: branch_child,
            } => {
                let shared = (&point ^ child_point).leading_zeros();
                if shared >= *point_depth {
                    // They share all their data (up to point depth)
                    if let Some(new) = self.add_to_branch(*branch_child, data, point, *point_depth)
                    {
                        self.set_skip_child(branch, new);
                    }
                    None
                } else {
                    let child_point = child_point.clone();
                    let new = self.add_branch(Branch::new_data(point.clone(), data));
                    Some(self.add_new_split(new, branch, point, &child_point, shared, depth))
                }
            }
            Branch::Split { children, .. } => {
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
        point1: PointData<P>,
        point2: &PointData<P>,
        shared: u8,
        depth: u8,
    ) -> BranchKey {
        let dir1 = point1.nth(shared) as usize;
        let dir2 = point2.nth(shared) as usize;
        let mut children = [None, None, None, None, None, None, None, None];
        children[dir1] = Some(child1);
        children[dir2] = Some(child2);
        let split = self.add_branch(Branch::Split {
            children,
            occupied: 2,
            depth: shared + 1,
        });

        if shared > depth {
            self.add_branch(Branch::Skip {
                point: point1,
                point_depth: shared,
                child: split,
            })
        } else {
            split
        }
    }

    /// Sets the child of the given branch to 'new' if it is a skip branch (N.B. must be passed a skip branch)
    fn set_skip_child(&mut self, branch: BranchKey, new: BranchKey) {
        let Branch::Skip { child, .. } = self.get_branch_mut(branch) else {
            unreachable!()
        };
        *child = new;
    }

    /// Sets a child of the given branch to 'new' if it is a split branch (N.B. must be passed a split branch)
    fn set_split_child(&mut self, branch: BranchKey, ind: usize, new: BranchKey) {
        let Branch::Split {
            children, occupied, ..
        } = self.get_branch_mut(branch)
        else {
            unreachable!()
        };
        if children[ind].is_none() {
            *occupied += 1;
        }
        children[ind] = Some(new);
    }
}
