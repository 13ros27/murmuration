use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use murmuration_octree::{Point, PointData};
use std::marker::PhantomData;

use crate::SpatialTree;

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
pub struct SpatialPlugin<P: Component + Point>(PhantomData<P>)
where
    P::Data: Send + Sync;

impl<P> SpatialPlugin<P>
where
    P: Component + Point,
    P::Data: Send + Sync,
{
    /// Create a new `SpatialPlugin<P>`
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P> Default for SpatialPlugin<P>
where
    P: Component + Point,
    P::Data: Send + Sync,
{
    /// Create a new `SpatialPlugin<P>`
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<P> Plugin for SpatialPlugin<P>
where
    P: Component + Point,
    P::Data: Send + Sync,
{
    fn build(&self, app: &mut App) {
        // This is required to fulfil the safety invariants for SpatialMutIter
        assert!(
            app.world.get_resource::<SpatialTree<P>>().is_none(),
            "Setting up a SpatialPlugin for a component that already has a SpatialTree is invalid \
            as it would result in duplicated entities in the tree."
        );
        app.init_resource::<SpatialTree<P>>();

        // Add an entity to the spatial tree when P gets added to it
        app.world.observer(
            |observer: Observer<OnAdd, P>,
             mut commands: Commands,
             query: Query<&P>,
             mut spatial: ResMut<SpatialTree<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.add(entity, point);
                commands
                    .entity(entity)
                    .try_insert(OldPosition(point.get_point()));
            },
        );
        // Add an entity to the spatial tree if P gets inserted to it and wasn't already there,
        // otherwise move the entity to its new position in the tree and update OldPosition.
        app.world.observer(
            |observer: Observer<OnInsert, P>,
             mut commands: Commands,
             mut query: Query<(&P, Option<&mut OldPosition<P>>)>,
             mut spatial: ResMut<SpatialTree<P>>| {
                let entity = observer.source();
                let (point, old_point) = query.get_mut(entity).unwrap();
                if let Some(mut old_point) = old_point {
                    spatial.move_entity(entity, &old_point.0, point.get_point());
                    *old_point = OldPosition(point.get_point());
                } else {
                    spatial.add(entity, point);
                    commands
                        .entity(entity)
                        .try_insert(OldPosition(point.get_point()));
                }
            },
        );
        // Remove an entity from the spatial tree when P gets removed from it (or it is despawned)
        app.world.observer(
            |observer: Observer<OnRemove, P>,
             mut commands: Commands,
             query: Query<&P>,
             mut spatial: ResMut<SpatialTree<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.remove(entity, point);
                commands.entity(entity).remove::<OldPosition<P>>();
            },
        );
    }
}

/// Automatically added to all entities with the component `P`.
///
/// This is publicly visible so that it can be used in [`SpatialTree::update_tree`].
#[derive(Component, Debug)]
#[doc(hidden)]
pub struct OldPosition<P: Point>(pub(crate) PointData<P>);
