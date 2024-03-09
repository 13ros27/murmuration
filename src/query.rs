use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryFilter, ROQueryItem},
    system::SystemParam,
};
use bevy_transform::components::Transform;

use crate::octree::point::Point;
use crate::{mut_iter::SpatialMutIter, SpatialGrid};

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
    ) -> SpatialMutIter<'w, 's, 'a, impl Iterator<Item = Entity> + 'a, D, (F, With<Transform>)>
    {
        // SAFETY: .get will never return the same element twice and the grid cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { SpatialMutIter::new(self.grid.get(point), &mut self.query) }
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
    ) -> SpatialMutIter<'w, 's, 'a, impl Iterator<Item = Entity> + 'a, D, (F, With<Transform>)>
    {
        // SAFETY: .within will never return the same element twice and the grid cannot contain
        //  duplicates (as only the observers can change it)
        unsafe { SpatialMutIter::new(self.grid.within(point, distance), &mut self.query) }
    }
}
