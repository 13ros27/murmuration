mod sealed {
    use bevy_ecs::{
        prelude::*,
        query::{QueryData, QueryFilter},
        system::QueryLens,
    };

    pub trait IntoQuery<Q: QueryData> {
        fn transmute_lens(&mut self) -> QueryLens<'_, Q>;
    }

    impl<Q: QueryData, D: QueryData, F: QueryFilter> IntoQuery<Q> for Query<'_, '_, D, F> {
        fn transmute_lens(&mut self) -> QueryLens<'_, Q> {
            self.transmute_lens::<Q>()
        }
    }
}
pub(crate) use sealed::IntoQuery;
