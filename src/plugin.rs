use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use murmuration_octree::Point;
use std::marker::PhantomData;

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
    /// Create a new `SpatialPlugin<P>`
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P: Component + Point> Default for SpatialPlugin<P> {
    /// Create a new `SpatialPlugin<P>`
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
             mut commands: Commands,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.add(entity, point);
                commands
                    .entity(entity)
                    .try_insert(OldPosition(point.clone()));
            },
        );
        // Add an entity to the spatial grid if P gets inserted to it and wasn't already there,
        // otherwise move the entity to its new position in the grid and update OldPosition.
        app.world.observer(
            |observer: Observer<OnInsert, P>,
             mut commands: Commands,
             mut query: Query<(&P, Option<&mut OldPosition<P>>)>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let (point, old_point) = query.get_mut(entity).unwrap();
                if let Some(mut old_point) = old_point {
                    spatial.move_entity(entity, &old_point.0, point);
                    *old_point = OldPosition(point.clone());
                } else {
                    spatial.add(entity, point);
                    commands
                        .entity(entity)
                        .try_insert(OldPosition(point.clone()));
                }
            },
        );
        // Remove an entity from the spatial grid when P gets removed from it (or it is despawned)
        app.world.observer(
            |observer: Observer<OnRemove, P>,
             mut commands: Commands,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.remove(entity, point);
                commands.entity(entity).remove::<OldPosition<P>>();
            },
        );
    }
}

pub(crate) mod sealed {
    use bevy_ecs::prelude::*;
    use murmuration_octree::Point;

    #[derive(Component, Debug)]
    pub struct OldPosition<P: Point>(pub(crate) P);
}
use sealed::*;
