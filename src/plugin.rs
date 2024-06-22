use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use murmuration_octree::{Point, PointData};
use std::marker::PhantomData;

use crate::SpatialTree;

mod sealed {
    use bevy::ecs::{prelude::*, query::QueryFilter};

    pub trait OptComponent: Send + Sync + 'static {
        type Bundle<P: Component>: Bundle;
        type QueryFilter: QueryFilter;
    }
    pub struct NoFilter;
    impl OptComponent for NoFilter {
        type Bundle<P: Component> = P;
        type QueryFilter = ();
    }
    impl<C: Component> OptComponent for C {
        type Bundle<P: Component> = (P, C);
        type QueryFilter = With<C>;
    }
}

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
///
/// You can also add a filter component with the `F` generic, in which case this will only add
/// entities with that component to the [`SpatialTree`].
pub struct SpatialPlugin<P: Component + Point, F: sealed::OptComponent = sealed::NoFilter>(
    PhantomData<(P, F)>,
);

impl<P: Component + Point, F: sealed::OptComponent> SpatialPlugin<P, F> {
    /// Create a new `SpatialPlugin<P, F>`
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P: Component + Point, F: sealed::OptComponent> Default for SpatialPlugin<P, F> {
    /// Create a new `SpatialPlugin<P, F>`
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<P: Component + Point, F: sealed::OptComponent> Plugin for SpatialPlugin<P, F> {
    fn build(&self, app: &mut App) {
        // This is required to fulfil the safety invariants for SpatialMutIter
        assert!(
            app.world().get_resource::<SpatialTree<P>>().is_none(),
            "Setting up a SpatialPlugin for a component that already has a SpatialTree is invalid \
            as it would result in duplicated entities in the tree."
        );
        app.init_resource::<SpatialTree<P>>();

        // Add an entity to the spatial tree if P gets inserted to it and wasn't already there,
        //  otherwise move the entity to its new position in the tree and update OldPosition.
        // This will also trigger if F is changed (despite it just being a filter) but if we move
        //  that to an OnAdd observer to prevent this we can get duplicate entities in the tree.
        app.world_mut().observe(
            |observer: Trigger<OnInsert, F::Bundle<P>>,
             mut commands: Commands,
             mut query: Query<(&P, Option<&mut OldPosition<P>>), F::QueryFilter>,
             mut spatial: ResMut<SpatialTree<P>>| {
                let entity = observer.entity();
                if let Ok((point, old_point)) = query.get_mut(entity) {
                    if let Some(mut old_point) = old_point {
                        spatial.move_entity(entity, &old_point.0, point.get_point());
                        *old_point = OldPosition(point.get_point());
                    } else {
                        spatial.add(entity, point);
                        commands
                            .entity(entity)
                            .try_insert(OldPosition(point.get_point()));
                    }
                }
            },
        );
        // Remove an entity from the spatial tree when P gets removed from it (or it is despawned)
        app.world_mut().observe(
            |trigger: Trigger<OnRemove, F::Bundle<P>>,
             mut commands: Commands,
             query: Query<&P, F::QueryFilter>,
             mut spatial: ResMut<SpatialTree<P>>| {
                let entity = trigger.entity();
                if let Ok(point) = query.get(entity) {
                    spatial.remove(entity, point);
                    commands.entity(entity).remove::<OldPosition<P>>();
                }
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
