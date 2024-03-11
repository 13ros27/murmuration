use bevy::{color::palettes::basic, prelude::*};
use rand::distributions::{Distribution, Uniform};

use murmuration::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpatialPlugin::<Transform>::new()))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2500.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, colour_enemies).chain())
        .run();
}

#[derive(Component)]
struct Player {
    last_position: Vec3,
}

#[derive(Component)]
struct Enemy;

#[derive(Resource, Deref)]
struct RedMaterial(Handle<StandardMaterial>);
#[derive(Resource, Deref)]
struct WhiteMaterial(Handle<StandardMaterial>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(100.0, 100.0, 100.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn((
        Player {
            last_position: Vec3::ZERO,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let red = materials.add(Color::from(basic::RED));
    commands.insert_resource(RedMaterial(red));
    let white = materials.add(Color::WHITE);
    commands.insert_resource(WhiteMaterial(white.clone()));

    let sphere = meshes.add(Sphere::new(0.5));

    let uniform = Uniform::new_inclusive(-100.0, 100.0);
    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
        commands.spawn((
            Enemy,
            PbrBundle {
                mesh: sphere.clone(),
                material: white.clone(),
                transform: Transform::from_xyz(
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                    uniform.sample(&mut rng),
                ),
                ..default()
            },
        ));
    }
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(&mut Transform, &mut Player)>,
) {
    let forward = input.pressed(KeyCode::KeyW) as isize - input.pressed(KeyCode::KeyS) as isize;
    let right = input.pressed(KeyCode::KeyD) as isize - input.pressed(KeyCode::KeyA) as isize;
    let up =
        input.pressed(KeyCode::ShiftLeft) as isize - input.pressed(KeyCode::ControlLeft) as isize;

    let (mut transform, mut player) = player.single_mut();
    let movement = Vec3::X * forward as f32 + Vec3::Z * right as f32 + Vec3::Y * up as f32;
    if movement != Vec3::ZERO {
        player.last_position = transform.translation;
        transform.translation += movement * 100.0 * time.delta_seconds();
    }
}

fn colour_enemies(
    player: Query<(&Transform, &Player)>,
    mut spatial: TransformQuery<&mut Handle<StandardMaterial>, With<Enemy>>,
    red: Res<RedMaterial>,
    white: Res<WhiteMaterial>,
) {
    let (transform, player) = player.single();
    for mut material in spatial.within_mut(&Transform::from_translation(player.last_position), 50.0)
    {
        *material = white.clone();
    }
    for mut material in spatial.within_mut(transform, 50.0) {
        *material = red.clone();
    }
}
