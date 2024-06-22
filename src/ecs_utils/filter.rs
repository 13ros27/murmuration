use bevy_ecs::{
    archetype::Archetype,
    component::{ComponentId, Components, Tick},
    prelude::*,
    query::{FilteredAccess, QueryData, QueryFilter, WorldQuery},
    storage::{Table, TableRow},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::marker::PhantomData;

#[derive(Default)]
pub struct Filter<D: QueryData>(PhantomData<D>);

unsafe impl<D: QueryData> WorldQuery for Filter<D> {
    type Item<'a> = ();
    type Fetch<'a> = ();
    type State = D::State;

    fn shrink<'wlong: 'wshort, 'wshort>(_item: Self::Item<'wlong>) -> Self::Item<'wshort> {}

    #[inline]
    unsafe fn init_fetch<'w>(
        _world: UnsafeWorldCell<'w>,
        _state: &Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Fetch<'w> {
    }

    const IS_DENSE: bool = D::IS_DENSE;

    #[inline]
    unsafe fn set_archetype<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _state: &Self::State,
        _archetype: &'w Archetype,
        _table: &'w Table,
    ) {
    }

    unsafe fn set_table<'w>(_fetch: &mut Self::Fetch<'w>, _state: &Self::State, _table: &'w Table) {
    }

    unsafe fn fetch<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        _table_row: TableRow,
    ) -> Self::Item<'w> {
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        D::update_component_access(state, access);
    }

    fn init_state(world: &mut World) -> Self::State {
        D::init_state(world)
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        D::get_state(components)
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        D::matches_component_set(state, set_contains_id)
    }
}

impl<D: QueryData> QueryFilter for Filter<D> {
    const IS_ARCHETYPAL: bool = true;
    unsafe fn filter_fetch(
        _fetch: &mut Self::Fetch<'_>,
        _entity: Entity,
        _table_row: TableRow,
    ) -> bool {
        // The same as With<T>
        true
    }
}
