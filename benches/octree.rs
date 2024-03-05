use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::UVec3;
use murmuration::octree::Octree;
use rand::distributions::{Distribution, Uniform};
use std::num::NonZeroU64;

criterion_group!(all, add_many, get_many);
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

    c.bench_function("get_single 100_000", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                black_box(tree.get_single(UVec3::new(
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                )));
            }
        })
    });
}
