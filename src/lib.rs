mod mut_iter;
pub mod octree;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy_ecs::system::{EntityCommands, SystemParam};
use bevy_transform::prelude::*;

use crate::octree::{point::Point, Octree};

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

pub type TransformQuery<'w, 's, D, F = ()> = SpatialQuery<'w, 's, Transform, D, F>;

#[derive(SystemParam)]
pub struct SpatialQuery<'w, 's, P, D, F = ()>
where
    P: Point + 'static,
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    grid: Res<'w, SpatialGrid<P>>,
    query: Query<'w, 's, D, (F, With<Transform>)>,
}

impl<'w, 's, P, D, F> SpatialQuery<'w, 's, P, D, F>
where
    P: Point,
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
    ) -> mut_iter::SpatialMutIter<
        'w,
        's,
        'a,
        impl Iterator<Item = Entity> + 'a,
        D,
        (F, With<Transform>),
    > {
        // SAFETY: .get will never return the same element twice and the grid cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { mut_iter::SpatialMutIter::new(self.grid.get(point), &mut self.query) }
    }

    pub fn within(&self, point: &P, distance: P::Data) -> impl Iterator<Item = ROQueryItem<'_, D>> {
        self.grid
            .within(point, distance)
            .filter_map(|e| self.query.get(e).ok())
    }

    pub fn within_mut<'a>(
        &'a mut self,
        point: &P,
        distance: P::Data,
    ) -> mut_iter::SpatialMutIter<
        'w,
        's,
        'a,
        impl Iterator<Item = Entity> + 'a,
        D,
        (F, With<Transform>),
    > {
        // SAFETY: .within will never return the same element twice and the grid cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { mut_iter::SpatialMutIter::new(self.grid.within(point, distance), &mut self.query) }
    }
}

#[derive(Component)]
struct SpatialMove<P: Point + Send + Sync>(P);

#[derive(Resource)]
pub struct SpatialGrid<P: Point>(Octree<Entity, P>);

impl<P: Point> Default for SpatialGrid<P> {
    fn default() -> Self {
        Self(Octree::default())
    }
}

impl<P: Point> SpatialGrid<P> {
    fn add(&mut self, entity: Entity, point: P) {
        self.0.add(point, entity);
    }

    fn remove(&mut self, entity: &Entity, point: P) -> bool {
        self.0.remove(point, entity)
    }

    fn move_entity(&mut self, entity: Entity, old_point: P, new_point: P) -> bool {
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

impl Point for Transform {
    type Data = f32;
    fn to_array(&self) -> [f32; 3] {
        self.translation.to_array()
    }
}

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
