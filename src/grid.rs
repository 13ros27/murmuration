use bevy_ecs::prelude::*;

use crate::octree::{point::Point, Octree};

#[derive(Resource)]
pub struct SpatialGrid<P: Point>(Octree<Entity, P>);

impl<P: Point> Default for SpatialGrid<P> {
    fn default() -> Self {
        Self(Octree::default())
    }
}

impl<P: Point> SpatialGrid<P> {
    pub(crate) fn add(&mut self, entity: Entity, point: P) {
        self.0.add(point, entity);
    }

    pub(crate) fn remove(&mut self, entity: &Entity, point: P) -> bool {
        self.0.remove(point, entity)
    }

    pub(crate) fn move_entity(&mut self, entity: Entity, old_point: P, new_point: P) -> bool {
        self.0.move_data(old_point, new_point, entity)
    }

    pub fn get_single(&self, point: &P) -> Option<Entity> {
        self.get(point).next()
    }

    pub fn get(&self, point: &P) -> impl Iterator<Item = Entity> + '_ {
        self.0.get(point).copied()
    }

    pub fn within(&self, point: &P, distance: P::Data) -> impl Iterator<Item = Entity> + '_ {
        self.0.within(point, distance).copied()
    }
}
