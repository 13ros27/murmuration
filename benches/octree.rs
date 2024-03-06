use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::UVec3;
use murmuration::octree::Octree;
use rand::distributions::{Distribution, Uniform};
use std::num::NonZeroU64;

criterion_group!(
    all,
    add_many,
    add_many_spatialtree,
    get_many,
    get_many_spatialtree
);
criterion_main!(all);

fn add_many(c: &mut Criterion) {
    let mut tree = Octree::new();
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();

    c.bench_function("add 100_000", |b| {
        b.iter(|| {
            for i in 0..100_000 {
                tree.add(
                    UVec3::new(
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

fn get_many(c: &mut Criterion) {
    let mut tree = Octree::new();
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();
    for i in 0..100_000 {
        tree.add(
            UVec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            ),
            NonZeroU64::new(i + 1).unwrap(),
        )
    }

    c.bench_function("get_single", |b| {
        b.iter(|| {
            black_box(tree.get_single(UVec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            )));
        })
    });
}

fn add_many_spatialtree(c: &mut Criterion) {
    let mut tree = spatialtree::OctTree::new();
    let uniform = Uniform::new_inclusive(0, u32::MAX);
    let mut rng = rand::thread_rng();

    c.bench_function("spatial_tree add 100_000", |b| {
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

fn get_many_spatialtree(c: &mut Criterion) {
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
