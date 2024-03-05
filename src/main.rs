use glam::UVec3;
use murmuration::octree::Octree;
use std::num::NonZeroU64;

fn main() {
    let mut tree = Octree::new();
    tree.add(
        UVec3::new(0b11111111111111111111111111111111, 0b001, 0b101),
        NonZeroU64::new(1).unwrap(),
    );
    tree.add(
        UVec3::new(0b01111111111111111111111111111111, 0b001, 0b111),
        NonZeroU64::new(2).unwrap(),
    );
    tree.add(
        UVec3::new(
            0b01111111111111111111111111111111,
            0b10000000000000000000000000000001,
            0b101,
        ),
        NonZeroU64::new(3).unwrap(),
    );
    tree.add(UVec3::new(0b01, 0, 0), NonZeroU64::new(4).unwrap());
    tree.add(UVec3::new(0b10, 0, 0), NonZeroU64::new(5).unwrap());
    tree.add(UVec3::new(0b11, 0, 0), NonZeroU64::new(6).unwrap());
    tree.add(UVec3::new(0b10, 0b10, 0), NonZeroU64::new(7).unwrap());
    println!("{:?}\n{:?}", tree, tree.get_single(UVec3::new(0b10, 0, 0)));
}
