use bevy::ecs::{
    prelude::*,
    query::{QueryData, QueryFilter, ROQueryItem},
    system::SystemParam,
};
use bevy::transform::components::Transform;
use fix_hidden_lifetime_bug::Captures;
use murmuration_octree::Point;

#[cfg(feature = "change_detection")]
use {
    crate::{ecs_utils::filter::Filter, plugin::OldPosition},
    bevy::ecs::{
        archetype::Archetype, component::Tick, system::SystemMeta,
        world::unsafe_world_cell::UnsafeWorldCell,
    },
};

use crate::{mut_iter::SpatialMutIter, SpatialTree};

/// An alias for `SpatialQuery<Transform, ..>`
pub type TransformQuery<'w, 's, D, F = ()> = SpatialQuery<'w, 's, Transform, D, F>;

/// A system parameter for easy spatial querying.
///
/// The first generic specifies what type the spatial tree is defined over and is autofilled to
/// [`Transform`](`bevy::prelude::Transform`) by [`TransformQuery`]. The other two are the
/// same as the data and filter types on [`Query`].
#[cfg_attr(not(feature = "change_detection"), derive(SystemParam))]
pub struct SpatialQuery<'w, 's, P, D, F = ()>
where
    P: Component + Point + 'static,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    tree: Res<'w, SpatialTree<P>>,
    query: Query<'w, 's, D, (F, With<Transform>)>,
}

impl<'w, 's, P, D, F> SpatialQuery<'w, 's, P, D, F>
where
    P: Component + Point,
    D: QueryData,
    F: QueryFilter,
{
    /// Returns an immutable reference to the underlying [`Query`].
    pub fn as_query(&self) -> &Query<'w, 's, D, (F, With<Transform>)> {
        &self.query
    }

    /// Returns a mutable reference to the underlying [`Query`].
    pub fn as_query_mut(&mut self) -> &mut Query<'w, 's, D, (F, With<Transform>)> {
        &mut self.query
    }

    /// Returns an [`Iterator`] over the read-only query items at the given point.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use murmuration::TransformQuery;
    /// # #[derive(Component)]
    /// # struct Player;
    /// #[derive(Component)]
    /// struct Enemy {
    ///     name: String,
    /// }
    ///
    /// /// Print the names of all enemies on top of the player
    /// fn print_enemy_names(player: Query<&Transform, With<Player>>, spatial: TransformQuery<&Enemy>) {
    ///     for enemy in spatial.get(player.single()) {
    ///         println!("The enemy '{}' is being very rude and sitting on me.", enemy.name);
    ///     }
    /// }
    /// ```
    /// # See also
    /// - [`get_mut`](Self::get_mut) for mutable queries
    /// - [`within`](Self::within) to get all within a radius rather than only at the point exactly
    pub fn get(&self, point: &P) -> impl Iterator<Item = ROQueryItem<'_, D>> {
        self.tree.get(point).filter_map(|e| self.query.get(e).ok())
    }

    /// Returns an [`Iterator`] over the query items at the given point.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use murmuration::TransformQuery;
    /// # #[derive(Component)]
    /// # struct Player;
    /// #[derive(Component)]
    /// struct Enemy {
    ///     name: String,
    /// }
    ///
    /// /// Rename all enemies in the same position as the player to 'Rude'
    /// fn rename_nearby_enemies(
    ///     player: Query<&Transform, With<Player>>,
    ///     mut spatial: TransformQuery<&mut Enemy>
    /// ) {
    ///     for mut enemy in spatial.get_mut(player.single()) {
    ///         enemy.name = "Rude".to_string();
    ///     }
    /// }
    /// ```
    /// # See also
    /// - [`get`](Self::get) for immutable queries
    /// - [`within_mut`](Self::within_mut) to get all within a radius rather than only at the point
    /// exactly
    pub fn get_mut(
        &mut self,
        point: &P,
    ) -> impl Iterator<Item = D::Item<'_>> + Captures<'w> + Captures<'s> {
        // SAFETY: .get will never return the same element twice and the tree cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { SpatialMutIter::new(self.tree.get(point), &mut self.query) }
    }

    /// Returns an [`Iterator`] over the read-only query items that are within `distance` of the given
    /// point.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
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
        self.tree
            .within(point, distance)
            .filter_map(|e| self.query.get(e).ok())
    }

    /// Returns an [`Iterator`] over the query items that are within `distance` of the given point.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
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
    pub fn within_mut(
        &mut self,
        point: &P,
        distance: P::Data,
    ) -> impl Iterator<Item = D::Item<'_>> + Captures<'w> + Captures<'s> {
        // SAFETY: .within will never return the same element twice and the tree cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { SpatialMutIter::new(self.tree.within(point, distance), &mut self.query) }
    }
}

#[cfg(feature = "change_detection")]
type SpatialQuerySet<P, D, F> = (
    ResMut<'static, SpatialTree<P>>,
    ParamSet<
        'static,
        'static,
        (
            Query<'static, 'static, D, (F, With<Transform>)>,
            Query<
                'static,
                'static,
                (Entity, &'static P, &'static mut OldPosition<P>),
                (Filter<D>, F),
            >,
        ),
    >,
);

#[cfg(feature = "change_detection")]
unsafe impl<P, D, F> SystemParam for SpatialQuery<'_, '_, P, D, F>
where
    P: Component + Point + 'static,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    type State = <SpatialQuerySet<P, D, F> as SystemParam>::State;
    type Item<'world, 'state> = SpatialQuery<'world, 'state, P, D, F>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        SpatialQuerySet::<P, D, F>::init_state(world, system_meta)
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        // SAFETY: We pass all safety invariants from the inner call to this function
        unsafe {
            SpatialQuerySet::<P, D, F>::new_archetype(state, archetype, system_meta);
        }
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        // SAFETY: The tuple implementation upholds the safety invariants
        let (mut tree, mut param_set) = unsafe {
            SpatialQuerySet::<P, D, F>::get_param(state, system_meta, world, change_tick)
        };

        // Find any updated positions and update them in the tree
        let mut query = param_set.p1();
        for (entity, position, mut old_position) in &mut query {
            let pos_data = position.get_point();
            if pos_data != old_position.0 {
                tree.move_entity(entity, &old_position.0, pos_data.clone());
                *old_position = OldPosition(pos_data);
            }
        }

        // I believe this is sound because it is just working around how ParamSet uses a unique reference
        let query: Query<'world, 'state, _, _> = unsafe { std::mem::transmute(param_set.p0()) };
        SpatialQuery {
            tree: tree.into(),
            query,
        }
    }
}
