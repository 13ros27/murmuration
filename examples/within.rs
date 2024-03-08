use bevy::prelude::*;
use murmuration::{SpatialPlugin, TransformQuery};

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
    commands.spawn((Enemy, Transform::from_translation(Vec3::new(1.0, 5.0, 1.0))));
}

fn query_system(
    player: Query<&Transform, With<Player>>,
    spatial: TransformQuery<Entity, With<Enemy>>,
) {
    println!(
        "Within 5: {:?}",
        spatial
            .within(player.get_single().unwrap(), 6.0)
            .collect::<Vec<_>>()
    );
}
