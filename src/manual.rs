use bevy_ecs::prelude::*;
use murmuration_octree::Point;

use crate::{OldPosition, SpatialTree};

/// Exposes the [`update_tree`](Self::update_tree) method on [`&mut World`](World).
pub trait WorldExt {
    /// Updates the spatial tree for P with any changes since it was last updated.
    ///
    /// For more details see [`SpatialTree::update_tree`] although this doesn't allow filtering,
    /// instead just updating all entities with the component P.
    fn update_tree<P: Component + Point>(&mut self);
}

impl WorldExt for World {
    fn update_tree<P: Component + Point>(&mut self) {
        self.resource_scope(|world, mut tree: Mut<SpatialTree<P>>| {
            for (entity, position, mut old_position) in world
                .query::<(Entity, &P, &mut OldPosition<P>)>()
                .iter_mut(world)
            {
                let pos_data = position.get_point();
                if pos_data != old_position.0 {
                    tree.move_entity(entity, &old_position.0, pos_data.clone());
                    *old_position = OldPosition(pos_data);
                }
            }
        });
    }
}

/// A system for updating the spatial tree with all changes since it was last updated.
///
/// This is useful when manually using [`Res<SpatialTree>`](SpatialTree) as that won't do the update
/// for you, [`SpatialQuery`](crate::SpatialQuery) doesn't need this.
///
/// See also: [`World::update_tree`](WorldExt::update_tree)
pub fn update_spatial_tree<P: Component + Point>(world: &mut World) {
    world.update_tree::<P>();
}
