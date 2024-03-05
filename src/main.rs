use glam::UVec3;
use murmuration::octree::Octree;

fn main() {
    let mut tree = Octree::new();
    tree.add(
        UVec3::new(0b11111111111111111111111111111111, 0b001, 0b101),
        1,
    );
    tree.add(
        UVec3::new(0b01111111111111111111111111111111, 0b001, 0b111),
        2,
    );
    tree.add(
        UVec3::new(
            0b01111111111111111111111111111111,
            0b10000000000000000000000000000001,
            0b101,
        ),
        3,
    );
    tree.add(UVec3::new(0b01, 0, 0), 4);
    tree.add(UVec3::new(0b10, 0, 0), 5);
    tree.add(UVec3::new(0b11, 0, 0), 6);
    tree.add(UVec3::new(0b10, 0b10, 0), 7);
    println!("{:?}\n{:?}", tree, tree.get_single(UVec3::new(0b10, 0, 0)));
}
