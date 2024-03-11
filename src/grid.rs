use bevy_ecs::prelude::*;
use murmuration_octree::{Octree, Point};

/// A resource storing the spatial grid for the component `P`.
///
/// Created by [`SpatialPlugin`](crate::SpatialGrid) this can be used to directly get the entities
/// for a particular spatial query rather than going through [`SpatialQuery`](crate::SpatialQuery).
#[derive(Resource)]
pub struct SpatialGrid<P: Component + Point>(Octree<Entity, P>);

impl<P: Component + Point> Default for SpatialGrid<P> {
    fn default() -> Self {
        Self(Octree::default())
    }
}

impl<P: Component + Point> SpatialGrid<P> {
    pub(crate) fn add(&mut self, entity: Entity, point: &P) {
        self.0.add(point, entity);
    }

    pub(crate) fn remove(&mut self, entity: Entity, point: &P) -> bool {
        self.0.remove(point, &entity)
    }

    pub(crate) fn move_entity(&mut self, entity: Entity, old_point: &P, new_point: &P) -> bool {
        self.0.move_data(old_point, new_point, entity)
    }

    /// Returns the entity at the given point or `None` if there is nothing there.
    ///
    /// If there could be multiple entities in the same location you may want to use
    /// [`get`](Self::get) instead.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::Vec3;
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_transform::prelude::*;
    /// # use murmuration::SpatialGrid;
    /// /// Prints one of the entities at (0, 0, 0)
    /// fn on_centre_system(grid: Res<SpatialGrid<Transform>>) {
    ///     println!("{:?}", grid.get_single(&Transform::from_xyz(0.0, 0.0, 0.0)));
    /// }
    /// ```
    pub fn get_single(&self, point: &P) -> Option<Entity> {
        self.get(point).next()
    }

    /// Returns all the entities at the given `point`.
    ///
    /// If there is some small variation around the point, you may want to use [`within`](Self::within) with
    /// a small distance instead.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::Vec3;
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_transform::prelude::*;
    /// # use murmuration::SpatialGrid;
    /// /// Prints all the entities at exactly (0, 0, 0)
    /// fn on_centre_system(grid: Res<SpatialGrid<Transform>>) {
    ///     for entity in grid.get(&Transform::from_xyz(0.0, 0.0, 0.0)) {
    ///         println!("{:?}", entity);
    ///     }
    /// }
    /// ```
    pub fn get(&self, point: &P) -> impl Iterator<Item = Entity> + '_ {
        self.0.get(point).copied()
    }

    /// Returns all the entities within a radius `distance` of the given `point`.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::Vec3;
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_transform::prelude::*;
    /// # use murmuration::SpatialGrid;
    /// /// Prints all the entities within 10 of (0, 0, 0)
    /// fn near_centre_system(grid: Res<SpatialGrid<Transform>>) {
    ///     for entity in grid.within(&Transform::from_xyz(0.0, 0.0, 0.0), 10.0) {
    ///         println!("{:?}", entity);
    ///     }
    /// }
    /// ```
    pub fn within(&self, point: &P, distance: P::Data) -> impl Iterator<Item = Entity> + '_ {
        self.0.within(point, distance).copied()
    }
}
