use bevy_app::{App, Plugin};
use bevy_ecs::{prelude::*, system::EntityCommands};
use std::marker::PhantomData;

use crate::octree::point::Point;
use crate::SpatialGrid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpatialPlugin<P: Component + Point>(PhantomData<P>);

impl<P: Component + Point> SpatialPlugin<P> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<P: Component + Point> Default for SpatialPlugin<P> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<P: Component + Point> Plugin for SpatialPlugin<P> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialGrid<P>>();
        app.world.init_component::<SpatialMove<P>>();

        // Add an entity to the spatial grid when P gets added to it
        app.world.observer(
            |observer: Observer<OnAdd, P>,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.add(entity, point.clone());
            },
        );
        // Remove an entity from the spatial grid when P gets removed from it (or it is despawned)
        app.world.observer(
            |observer: Observer<OnRemove, P>,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.remove(&entity, point.clone());
            },
        );
        // Move an entity when we trigger a SpatialMove event on it
        app.world.observer(
            |observer: Observer<SpatialMove<P>>,
             mut query: Query<&mut P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let SpatialMove(new_point) = observer.data();
                if let Ok(mut point) = query.get_mut(entity) {
                    spatial.move_entity(entity, point.clone(), new_point.clone());
                    *point = new_point.clone();
                }
            },
        );
    }
}

#[derive(Component)]
struct SpatialMove<P: Point + Send + Sync>(P);

pub trait EntityCommandsExt {
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: P);
}

impl EntityCommandsExt for EntityCommands<'_> {
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: P) {
        let entity = self.id();
        self.commands()
            .event(SpatialMove(new_point))
            .entity(entity)
            .emit();
    }
}
