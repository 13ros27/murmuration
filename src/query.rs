use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryFilter, ROQueryItem},
    system::SystemParam,
};
use bevy_transform::components::Transform;

use crate::octree::point::Point;
use crate::{mut_iter::SpatialMutIter, SpatialGrid};

/// An alias for `SpatialQuery<Transform, ..>`
pub type TransformQuery<'w, 's, D, F = ()> = SpatialQuery<'w, 's, Transform, D, F>;

/// A system parameter for easy spatial querying.
#[derive(SystemParam)]
pub struct SpatialQuery<'w, 's, P, D, F = ()>
where
    P: Component + Point + 'static,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    grid: Res<'w, SpatialGrid<P>>,
    query: Query<'w, 's, D, (F, With<Transform>)>,
}

impl<'w, 's, P, D, F> SpatialQuery<'w, 's, P, D, F>
where
    P: Component + Point,
    D: QueryData,
    F: QueryFilter,
{
    pub fn as_query(&self) -> &Query<'w, 's, D, (F, With<Transform>)> {
        &self.query
    }

    pub fn as_query_mut(&mut self) -> &mut Query<'w, 's, D, (F, With<Transform>)> {
        &mut self.query
    }

    pub fn get(&self, point: &P) -> impl Iterator<Item = ROQueryItem<'_, D>> {
        self.grid.get(point).filter_map(|e| self.query.get(e).ok())
    }

    pub fn get_mut<'a>(
        &'a mut self,
        point: &P,
    ) -> SpatialMutIter<'w, 's, 'a, impl Iterator<Item = Entity> + 'a, D, (F, With<Transform>)>
    {
        // SAFETY: .get will never return the same element twice and the grid cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { SpatialMutIter::new(self.grid.get(point), &mut self.query) }
    }

    /// Returns an iterator over the read-only query items that are within `distance` of the given
    /// point.
    ///
    /// # Example
    /// ```
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_transform::prelude::*;
    /// # use murmuration::TransformQuery;
    /// # #[derive(Component)]
    /// # struct Player;
    /// #[derive(Component)]
    /// struct Enemy {
    ///     name: String,
    /// }
    ///
    /// /// Print the names of all enemies near the player
    /// fn print_enemy_names(player: Query<&Transform, With<Player>>, spatial: TransformQuery<&Enemy>) {
    ///     for enemy in spatial.within(player.single(), 10.0) {
    ///         println!("There is a nearby enemy called '{}'", enemy.name);
    ///     }
    /// }
    /// ```
    /// # See also
    /// - [`within_mut`](Self::within_mut) for mutable queries
    pub fn within(&self, point: &P, distance: P::Data) -> impl Iterator<Item = ROQueryItem<'_, D>> {
        self.grid
            .within(point, distance)
            .filter_map(|e| self.query.get(e).ok())
    }

    /// Returns an iterator over the query items that are within `distance` of the given point.
    ///
    /// # Example
    /// ```
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_transform::prelude::*;
    /// # use murmuration::TransformQuery;
    /// # #[derive(Component)]
    /// # struct Player;
    /// #[derive(Component)]
    /// struct Enemy {
    ///     name: String,
    /// }
    ///
    /// /// Rename all enemies near the player to 'Personal Space Ignorer'
    /// fn rename_nearby_enemies(
    ///     player: Query<&Transform, With<Player>>,
    ///     mut spatial: TransformQuery<&mut Enemy>
    /// ) {
    ///     for mut enemy in spatial.within_mut(player.single(), 10.0) {
    ///         enemy.name = "Personal Space Ignorer".to_string();
    ///     }
    /// }
    /// ```
    /// # See also
    /// - [`within`](Self::within) for immutable queries
    pub fn within_mut<'a>(
        &'a mut self,
        point: &P,
        distance: P::Data,
    ) -> SpatialMutIter<'w, 's, 'a, impl Iterator<Item = Entity> + 'a, D, (F, With<Transform>)>
    {
        // SAFETY: .within will never return the same element twice and the grid cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { SpatialMutIter::new(self.grid.within(point, distance), &mut self.query) }
    }
}
