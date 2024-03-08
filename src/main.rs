use glam::Vec3;
use std::num::NonZeroU64;

use murmuration::octree::Octree;

fn main() {
    let mut tree = Octree::new();
    tree.add(Vec3::new(13.0, 15.0, 7.0), NonZeroU64::new(1).unwrap());
    tree.add(Vec3::new(4.0, 0.0, 0.0), NonZeroU64::new(2).unwrap());
    tree.add(Vec3::new(-1.0, 0.0, 0.0), NonZeroU64::new(4).unwrap());
    tree.add(Vec3::new(2.0, 0.0, 0.0), NonZeroU64::new(5).unwrap());
    tree.add(Vec3::new(3.0, 0.0, 0.0), NonZeroU64::new(6).unwrap());
    tree.add(Vec3::new(2.0, 2.0, 0.0), NonZeroU64::new(7).unwrap());
    tree.add(Vec3::new(2.0, 2.0, 0.0), NonZeroU64::new(77).unwrap());
    println!(
        "{:?}\n{:?}",
        tree,
        tree.within(Vec3::new(2.0, 1.0, 0.0), 3.2)
            .collect::<Vec<_>>()
    );
}
