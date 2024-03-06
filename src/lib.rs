pub mod octree;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use std::marker::PhantomData;

use crate::octree::{point::Point, Octree};

#[ghost::phantom]
pub struct SpatialPlugin<P: Component + Point>;

impl<P: Component + Point> Plugin for SpatialPlugin<P> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialGrid<P>>();
        app.world.observer(
            |observer: Observer<OnAdd, P>,
             (query, mut spatial): (Query<&P>, ResMut<SpatialGrid<P>>)| {
                let entity = observer.source();
                let point = query.get(entity).unwrap(); // This should be fine because observer
                spatial.add(entity, point.clone());
            },
        );
    }
}

#[derive(Resource)]
struct SpatialGrid<T: Point>(Octree<Entity, T>, PhantomData<T>);

impl<P: Point> Default for SpatialGrid<P> {
    fn default() -> Self {
        Self(Octree::default(), PhantomData)
    }
}

impl<P: Point> SpatialGrid<P> {
    fn add(&mut self, entity: Entity, point: P) {
        self.0.add(point, entity);
    }
}
