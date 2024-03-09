use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_ecs::system::EntityCommands;

use crate::octree::point::Point;
use crate::SpatialGrid;

#[ghost::phantom]
pub struct SpatialPlugin<P: Component + Point>;

impl<P: Component + Point> Plugin for SpatialPlugin<P> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialGrid<P>>();
        app.world.init_component::<SpatialMove<P>>();
        app.world.observer(
            |observer: Observer<OnAdd, P>,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.add(entity, point.clone());
            },
        );
        app.world.observer(
            |observer: Observer<OnRemove, P>,
             query: Query<&P>,
             mut spatial: ResMut<SpatialGrid<P>>| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.remove(&entity, point.clone());
            },
        );
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

pub trait MoveToExt {
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: P);
}

impl MoveToExt for EntityCommands<'_> {
    fn move_to<P: Component + Point + Send + Sync + 'static>(&mut self, new_point: P) {
        let entity = self.id();
        self.commands()
            .event(SpatialMove(new_point))
            .entity(entity)
            .emit();
    }
}
