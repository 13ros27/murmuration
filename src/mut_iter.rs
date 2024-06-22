use bevy::ecs::{
    prelude::*,
    query::{QueryData, QueryFilter},
};

pub struct SpatialMutIter<'w, 's, 'a, I, D, F>
where
    I: Iterator<Item = Entity>,
    D: QueryData,
    F: QueryFilter,
{
    inner: I,
    query: &'a mut Query<'w, 's, D, F>,
}

impl<'w, 's, 'a, I, D, F> SpatialMutIter<'w, 's, 'a, I, D, F>
where
    I: Iterator<Item = Entity>,
    D: QueryData,
    F: QueryFilter,
{
    /// SAFETY: The iterator `inner` must never return the same Entity twice
    pub unsafe fn new(inner: I, query: &'a mut Query<'w, 's, D, F>) -> Self {
        Self { inner, query }
    }
}

impl<'s, 'a, I, D, F> Iterator for SpatialMutIter<'_, 's, 'a, I, D, F>
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
