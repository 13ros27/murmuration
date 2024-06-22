use glam::Vec3;
use murmuration_octree::Octree;
use std::num::NonZeroU64;

fn create_tree() -> Octree<NonZeroU64, Vec3> {
    let mut tree = Octree::new();
    tree.add(&Vec3::new(13.0, 15.0, 7.0), NonZeroU64::new(1).unwrap());
    tree.add(&Vec3::new(4.0, 0.0, 0.0), NonZeroU64::new(2).unwrap());
    tree.add(&Vec3::new(-1.0, 0.0, 0.0), NonZeroU64::new(4).unwrap());
    tree.add(&Vec3::new(2.0, 0.0, 0.0), NonZeroU64::new(5).unwrap());
    tree.add(&Vec3::new(3.0, 0.0, 0.0), NonZeroU64::new(6).unwrap());
    tree.add(&Vec3::new(2.0, 2.0, 0.0), NonZeroU64::new(7).unwrap());
    tree.add(&Vec3::new(2.0, 2.0, 0.0), NonZeroU64::new(77).unwrap());
    tree
}

#[test]
fn within_zero() {
    let tree = create_tree();
    assert_eq!(
        tree.within_new(&Vec3::new(2.0, 0.0, 0.0), 0.0)
            .collect::<Vec<_>>(),
        vec![&NonZeroU64::new(5).unwrap()]
    );
}

#[test]
fn within_2_5() {
    let tree = create_tree();
    assert_eq!(
        tree.within_new(&Vec3::new(2.0, 0.0, 0.0), 2.5)
            .collect::<Vec<_>>(),
        vec![
            &NonZeroU64::new(5).unwrap(),
            &NonZeroU64::new(6).unwrap(),
            &NonZeroU64::new(2).unwrap(),
            &NonZeroU64::new(77).unwrap(),
            &NonZeroU64::new(7).unwrap()
        ]
    );
}
