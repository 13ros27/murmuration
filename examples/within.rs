use bevy::prelude::*;
use murmuration::{SpatialPlugin, TransformQuery};
use rand::distributions::{Distribution, Uniform};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpatialPlugin::<Transform>))
        .add_systems(Startup, setup)
        .add_systems(Update, query_system)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

fn setup(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
    let uniform = Uniform::new_inclusive(-10_000.0, 10_000.0);
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        commands.spawn((
            Enemy,
            Transform::from_translation(Vec3::new(
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
                uniform.sample(&mut rng),
            )),
        ));
    }
}

fn query_system(
    player: Query<&Transform, With<Player>>,
    mut spatial: TransformQuery<Entity, With<Enemy>>,
) {
    println!(
        "Within 1000: {:?}",
        spatial
            .within(player.get_single().unwrap(), 1000.0)
            .collect::<Vec<_>>()
    );
}
