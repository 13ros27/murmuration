use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::*,
    system::{EntityCommand, EntityCommands},
};
use bevy_log::warn;
use std::marker::PhantomData;

use crate::octree::point::Point;
use crate::SpatialGrid;

/// Plugin for setting up a spatial tree tracking the component `P`.
///
/// This is typically used to track `Transform` with `SpatialPlugin::<Transform>::new()`, and
/// added to the app with
/// ```
/// # use bevy::DefaultPlugins;
/// # use bevy_app::prelude::*;
/// # use bevy_ecs::prelude::*;
/// # use bevy_transform::prelude::*;
/// # use murmuration::SpatialPlugin;
/// App::new().add_plugins((DefaultPlugins, SpatialPlugin::<Transform>::new()));
/// ```
pub struct SpatialPlugin<P: Component + Point>(PhantomData<P>);

impl<P: Component + Point> SpatialPlugin<P> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P: Component + Point> Default for SpatialPlugin<P> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<P: Component + Point> Plugin for SpatialPlugin<P> {
    fn build(&self, app: &mut App) {
        // This is required to fulfil the safety invariants for SpatialMutIter
        assert!(
            app.world.get_resource::<SpatialGrid<P>>().is_none(),
            "Setting up a SpatialPlugin for a component that already has a SpatialGrid is invalid \
            as it would result in duplicated entities in the tree."
        );
        app.init_resource::<SpatialGrid<P>>();

        // Add an entity to the spatial grid when P gets added to it
        app.world.observer(
            |observer: Observer<OnAdd, P>,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.add(entity, point);
            },
        );
        // Remove an entity from the spatial grid when P gets removed from it (or it is despawned)
        app.world.observer(
            |observer: Observer<OnRemove, P>,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.remove(entity, point);
            },
        );
    }
}

/// Exposes the [`move_to`](Self::move_to) method on [`EntityWorldMut`](EntityWorldMut).
pub trait EntityWorldMutExt {
    /// This will change the given component on this entity to its new value.
    ///
    /// Importantly this will also update the [`SpatialGrid`](SpatialGrid) associated with
    /// this component, so this and [`EntityCommands::move_to`](EntityCommandsExt::move_to) are the
    /// only ways you should update the value of a tracked component.
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: &P);
}

impl EntityWorldMutExt for EntityWorldMut<'_> {
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: &P) {
        let point = if let Some(mut point) = self.get_mut() {
            *point = new_point.clone();
            point.clone()
        } else {
            return;
        };

        let entity = self.id();
        self.world_scope(|w| {
            if let Some(mut spatial) = w.get_resource_mut::<SpatialGrid<P>>() {
                spatial.move_entity(entity, &point, new_point);
            } else {
                warn!("Tried to move a non-spatial component using `move_to`");
            }
        });
    }
}

/// Exposes the [`move_to`](Self::move_to) method on [`EntityCommands`](EntityCommands).
pub trait EntityCommandsExt {
    /// This will queue up a change to the given component which occurs at the next sync point
    /// ([`apply_deferred`](`bevy_ecs::prelude::apply_deferred`)).
    ///
    /// Importantly this will also update the [`SpatialGrid`](SpatialGrid) associated with
    /// this component, so it is the only way you should update its value.
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: P);
}

impl EntityCommandsExt for EntityCommands<'_> {
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: P) {
        self.add(SpatialMove(new_point));
    }
}

struct SpatialMove<P: Component + Point + Send + Sync + 'static>(P);

impl<P: Component + Point + Send + Sync + 'static> EntityCommand for SpatialMove<P> {
    fn apply(self, id: Entity, world: &mut World) {
        world.entity_mut(id).move_to(&self.0);
    }
}
