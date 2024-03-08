use glam::UVec3;
use std::num::NonZeroU64;

use murmuration::octree::Octree;

fn main() {
    let mut tree = Octree::new();
    tree.add(UVec3::new(13, 15, 7), NonZeroU64::new(1).unwrap());
    tree.add(UVec3::new(4, 0, 0), NonZeroU64::new(2).unwrap());
    tree.add(UVec3::new(1, 0, 0), NonZeroU64::new(4).unwrap());
    tree.add(UVec3::new(2, 0, 0), NonZeroU64::new(5).unwrap());
    tree.add(UVec3::new(3, 0, 0), NonZeroU64::new(6).unwrap());
    tree.add(UVec3::new(2, 2, 0), NonZeroU64::new(7).unwrap());
    tree.add(UVec3::new(2, 2, 0), NonZeroU64::new(77).unwrap());
    // tree.remove(UVec3::new(13, 15, 7), &NonZeroU64::new(1).unwrap());
    // tree.remove(UVec3::new(2, 2, 0), &NonZeroU64::new(7).unwrap());
    // tree.remove(UVec3::new(1, 0, 0), &NonZeroU64::new(4).unwrap());
    // tree.remove(UVec3::new(4, 0, 0), &NonZeroU64::new(2).unwrap());
    // tree.remove(UVec3::new(3, 0, 0), &NonZeroU64::new(6).unwrap());
    // tree.remove(UVec3::new(2, 0, 0), &NonZeroU64::new(5).unwrap());
    println!(
        "{:?}\n{:?}",
        tree,
        tree.within(UVec3::new(4, 2, 0), 2).collect::<Vec<_>>()
    );
}
