use glam::UVec3;
use std::num::NonZeroU64;

use murmuration::octree::Octree;

fn main() {
    let mut tree = Octree::new();
    // tree.add(
    //     UVec3::new(0b11111111111111111111111111111111, 0b001, 0b101),
    //     NonZeroU64::new(1).unwrap(),
    // );
    // tree.add(
    //     UVec3::new(0b01111111111111111111111111111111, 0b001, 0b111),
    //     NonZeroU64::new(2).unwrap(),
    // );
    // tree.add(
    //     UVec3::new(
    //         0b01111111111111111111111111111111,
    //         0b10000000000000000000000000000001,
    //         0b101,
    //     ),
    //     NonZeroU64::new(3).unwrap(),
    // );
    tree.add(
        UVec3::new(0b1001010, 0b1000110, 0b1111111),
        NonZeroU64::new(1).unwrap(),
    );
    tree.add(UVec3::new(0b1000, 0b10, 0), NonZeroU64::new(2).unwrap());
    tree.add(UVec3::new(0b01, 0, 0), NonZeroU64::new(4).unwrap());
    tree.add(UVec3::new(0b10, 0, 0), NonZeroU64::new(5).unwrap());
    tree.add(UVec3::new(0b11, 0, 0), NonZeroU64::new(6).unwrap());
    tree.add(UVec3::new(0b10, 0b10, 0), NonZeroU64::new(7).unwrap());
    tree.add(UVec3::new(0b10, 0b10, 0), NonZeroU64::new(77).unwrap());
    println!(
        "{:?}\n{:?}",
        tree,
        tree.within(UVec3::new(0b10, 0b1, 0), 5000)
            .collect::<Vec<_>>()
    );
}
