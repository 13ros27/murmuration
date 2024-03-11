#![forbid(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]
//! A bevy plugin to track spatial indexes for your position type (typically
//! [`Transform`](bevy_transform::prelude::Transform)) making it easy and performant to query for
//! all entities at or near a point in space.
//!
//! ```
//! # use bevy::prelude::{DefaultPlugins, Time, Vec3};
//! # use bevy_app::prelude::*;
//! # use bevy_ecs::prelude::*;
//! # use bevy_transform::prelude::*;
//! # #[derive(Component)]
//! # struct Player;
//! # #[derive(Component)]
//! # struct Enemy { name: String }
//! use murmuration::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((DefaultPlugins, SpatialPlugin::<Transform>::new()))
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, (move_player, print_nearby));
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // Because this has a Transform and we added SpatialPlugin<Transform>, it will be tracked
//!     commands.spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0)));
//!
//!     // Spawn more entities with Transform ...
//! }
//!
//! fn move_player(
//!     mut query: Query<&Transform, With<Player>>,
//!     time: Res<Time>,
//! ) {
//!     let mut transform = query.single_mut();
//!     *transform.translation += Vec3::X * time.delta_seconds();
//! }
//!
//! fn print_nearby(player: Query<&Transform, With<Player>>, spatial: TransformQuery<&Enemy>) {
//!     for enemy in spatial.within(player.single(), 10.0) {
//!         println!("There is a nearby enemy called '{}'", enemy.name);
//!     }
//! }
//! ```

mod filter;
mod grid;
mod mut_iter;
mod plugin;
mod query;

pub use grid::SpatialGrid;
pub use plugin::SpatialPlugin;
pub use query::{SpatialQuery, TransformQuery};

/// Most commonly used re-exported types.
pub mod prelude {
    pub use super::{SpatialPlugin, TransformQuery};
}
