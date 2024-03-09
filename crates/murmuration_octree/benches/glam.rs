use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::{UVec3, Vec3};
use murmuration_octree::Octree;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::SliceRandom;
use std::num::NonZeroU64;

criterion_group!(
    all,
    add_many,
    add_many_spatialtree,
    get,
    get_spatialtree,
    within_many,
    remove_many,
    remove_many_spatialtree,
);
criterion_main!(all);

fn add_many(c: &mut Criterion) {
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();

    c.bench_function("add 100_000", |b| {
        let mut tree = Octree::new();
        b.iter(|| {
            for i in 0..100_000 {
                tree.add(
                    &UVec3::new(
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                    ),
                    NonZeroU64::new(i + 1).unwrap(),
                )
            }
        })
    });
}

fn get(c: &mut Criterion) {
    let mut tree = Octree::new();
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();
    for i in 0..100_000 {
        tree.add(
            &UVec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            ),
            NonZeroU64::new(i + 1).unwrap(),
        )
    }

    c.bench_function("get_single", |b| {
        b.iter(|| {
            black_box(tree.get_single(&UVec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            )));
        })
    });
}

fn remove_many(c: &mut Criterion) {
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();

    c.bench_function("remove_many", |b| {
        let mut tree = Octree::new();
        let mut items = Vec::new();
        for _ in 0..100_000 {
            let item = UVec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            );
            items.push(item);
            tree.add(&item, NonZeroU64::new(1).unwrap())
        }
        items.shuffle(&mut rng);
        b.iter(|| {
            black_box(for item in &items {
                tree.remove(item, &NonZeroU64::new(1).unwrap());
            });
        });
        assert_eq!(tree.num_branches(), 0);
    });
}

fn within_many(c: &mut Criterion) {
    let mut tree = Octree::new();
    // Has to be something like 30000 otherwise it is too easy to overflow
    let uniform = Uniform::new_inclusive(0, 30000);
    let mut rng = rand::thread_rng();
    for i in 0..100_000 {
        tree.add(
            &UVec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            ),
            NonZeroU64::new(i + 1).unwrap(),
        )
    }

    c.bench_function("within_1000", |b| {
        b.iter(|| {
            black_box(
                tree.within(
                    &UVec3::new(
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                    ),
                    1000,
                )
                .count(),
            );
        })
    });
    c.bench_function("within_10000", |b| {
        b.iter(|| {
            black_box(
                tree.within(
                    &UVec3::new(
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                    ),
                    10000,
                )
                .count(),
            );
        })
    });

    let mut tree = Octree::new();
    let uniform = Uniform::new_inclusive(-1000_000.0, 1000_000.0);
    let mut rng = rand::thread_rng();
    for i in 0..100_000 {
        tree.add(
            &Vec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            ),
            NonZeroU64::new(i + 1).unwrap(),
        )
    }

    c.bench_function("within_1000_f32", |b| {
        b.iter(|| {
            black_box(
                tree.within(
                    &Vec3::new(
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                        uniform.sample(&mut rng),
                    ),
                    1000.0,
                )
                .count(),
            );
        })
    });
}

fn add_many_spatialtree(c: &mut Criterion) {
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();

    c.bench_function("spatial_tree add 100_000", |b| {
        let mut tree = spatialtree::OctTree::new();
        b.iter(|| {
            for i in 0..100_000 {
                tree.insert(
                    spatialtree::CoordVec::new(
                        [
                            uniform.sample(&mut rng),
                            uniform.sample(&mut rng),
                            uniform.sample(&mut rng),
                        ],
                        32,
                    ),
                    |_| NonZeroU64::new(i + 1).unwrap(),
                );
            }
        })
    });
}

fn get_spatialtree(c: &mut Criterion) {
    let mut tree = spatialtree::OctTree::new();
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();
    for i in 0..100_000 {
        tree.insert(
            spatialtree::CoordVec::new(
                [
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                ],
                32,
            ),
            |_| NonZeroU64::new(i + 1).unwrap(),
        );
    }

    c.bench_function("spatial_tree get", |b| {
        b.iter(|| {
            black_box(tree.get_chunk_by_position(spatialtree::CoordVec::new(
                [
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                ],
                32,
            )))
        })
    });
}

fn remove_many_spatialtree(c: &mut Criterion) {
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();

    c.bench_function("spatialtree remove_many", |b| {
        let mut tree = spatialtree::OctTree::new();
        let mut items = Vec::new();
        for _ in 0..100_000 {
            let item = spatialtree::CoordVec::new(
                [
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                ],
                32,
            );
            items.push(item);
            tree.insert(item, |_| NonZeroU64::new(1).unwrap());
        }
        items.shuffle(&mut rng);
        b.iter(|| {
            black_box(for item in &items {
                tree.pop_chunk_by_position(*item);
            });
        });
    });
}
