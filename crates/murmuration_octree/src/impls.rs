#[cfg(feature = "bevy_transform")]
mod bevy_transform {
    use crate::point::Point;
    use bevy_transform::prelude::Transform;

    impl Point for Transform {
        type Data = f32;
        fn to_array(&self) -> [f32; 3] {
            self.translation.to_array()
        }
    }
}

#[cfg(feature = "glam")]
mod glam {
    use crate::point::Point;
    use glam::{IVec3, U64Vec3, UVec3, Vec3};

    impl Point for UVec3 {
        type Data = u32;
        fn to_array(&self) -> [u32; 3] {
            self.to_array()
        }
    }
    impl Point for IVec3 {
        type Data = i32;
        fn to_array(&self) -> [i32; 3] {
            self.to_array()
        }
    }
    impl Point for Vec3 {
        type Data = f32;
        fn to_array(&self) -> [f32; 3] {
            self.to_array()
        }
    }
    impl Point for U64Vec3 {
        type Data = u64;
        fn to_array(&self) -> [u64; 3] {
            self.to_array()
        }
    }
}
