pub mod octree;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy_ecs::system::{EntityCommand, EntityCommands, SystemParam};
use bevy_transform::prelude::*;

use crate::octree::{point::Point, Octree};

#[ghost::phantom]
pub struct SpatialPlugin<P: Component + Point>;

impl<P: Component + Point> Plugin for SpatialPlugin<P> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialGrid<P>>();
        app.world.register_component::<SpatialMove<P>>();
        app.world.observer(
            |observer: Observer<OnAdd, P>,
             (query, mut spatial): (Query<&P>, ResMut<SpatialGrid<P>>)| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.add(entity, point.clone());
            },
        );
        app.world.observer(
            |observer: Observer<OnRemove, P>,
             (query, mut spatial): (Query<&P>, ResMut<SpatialGrid<P>>)| {
                let entity = observer.source();
                let point = query.get(entity).unwrap();
                spatial.remove(&entity, point.clone());
            },
        );
        app.world.observer(
            |observer: Observer<SpatialMove<P>, P>,
             (mut query, mut spatial): (Query<&mut P>, ResMut<SpatialGrid<P>>)| {
                let entity = observer.source();
                let new_point = observer.data().0.clone();
                let mut point = query.get_mut(entity).unwrap();
                spatial.move_entity(entity, point.clone(), new_point.clone());
                *point = new_point;
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
    pub fn get(&self, point: &P) -> impl Iterator<Item = ROQueryItem<'_, D>> {
        self.grid.get(point).filter_map(|e| self.query.get(e).ok())
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
    ) -> sealed::SpatialMut<'w, 's, 'a, impl Iterator<Item = Entity> + 'a, D, (F, With<Transform>)>
    {
        sealed::SpatialMut {
            inner: self.grid.within(point, distance),
            query: &mut self.query,
        }
    }
}

mod sealed {
    use bevy_ecs::{
        prelude::*,
        query::{QueryData, QueryFilter},
    };

    pub struct SpatialMut<'w, 's, 'a, I, D, F>
    where
        I: Iterator<Item = Entity>,
        D: QueryData,
        F: QueryFilter,
    {
        pub(crate) inner: I,
        pub(crate) query: &'a mut Query<'w, 's, D, F>,
    }

    impl<'s, 'a, I, D, F> Iterator for SpatialMut<'_, 's, 'a, I, D, F>
    where
        I: Iterator<Item = Entity>,
        D: QueryData,
        F: QueryFilter,
    {
        type Item = D::Item<'a>;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(entity) = self.inner.next() {
                    let ptr = self.query as *mut Query<D, F>;
                    // SAFETY: The iterator in inner should always only return a particular Entity once
                    let prov_free_query = unsafe { ptr.as_mut().unwrap_unchecked() };
                    if let Ok(data) = prov_free_query.get_mut(entity) {
                        break Some(data);
                    }
                } else {
                    break None;
                }
            }
        }
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
    fn move_to<P: Point + Send + Sync + 'static>(&mut self, new_point: P);
}

impl MoveToExt for EntityWorldMut<'_> {
    fn move_to<P: Point + Send + Sync + 'static>(&mut self, new_point: P) {
        let entity = self.id();
        self.world_scope(|w| {
            w.ecs_event(SpatialMove(new_point)).entity(entity).emit();
        });
    }
}

impl MoveToExt for EntityCommands<'_> {
    fn move_to<P: Point + Send + Sync + 'static>(&mut self, new_point: P) {
        self.add(MoveToCommand(new_point));
    }
}

struct MoveToCommand<P: Point + Send + Sync + 'static>(P);

impl<P: Point + Send + Sync + 'static> EntityCommand for MoveToCommand<P> {
    fn apply(self, id: Entity, world: &mut World) {
        world.entity_mut(id).move_to(self.0);
    }
}
