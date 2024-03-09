mod grid;
mod mut_iter;
pub mod octree;
mod plugin;
mod query;

use bevy_transform::prelude::*;

use crate::octree::point::Point;

pub use grid::SpatialGrid;
pub use plugin::{MoveToExt, SpatialPlugin};
pub use query::{SpatialQuery, TransformQuery};

pub mod prelude {
    pub use super::{MoveToExt, SpatialPlugin, TransformQuery};
}

impl Point for Transform {
    type Data = f32;
    fn to_array(&self) -> [f32; 3] {
        self.translation.to_array()
    }
}
