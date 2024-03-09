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
    use glam::{DVec3, I16Vec3, I64Vec3, IVec3, U16Vec3, U64Vec3, UVec3, Vec3, Vec3A};

    impl Point for Vec3 {
        type Data = f32;
        fn to_array(&self) -> [f32; 3] {
            self.to_array()
        }
    }
    impl Point for Vec3A {
        type Data = f32;
        fn to_array(&self) -> [f32; 3] {
            self.to_array()
        }
    }
    impl Point for DVec3 {
        type Data = f64;
        fn to_array(&self) -> [f64; 3] {
            self.to_array()
        }
    }

    impl Point for U16Vec3 {
        type Data = u16;
        fn to_array(&self) -> [u16; 3] {
            self.to_array()
        }
    }
    impl Point for UVec3 {
        type Data = u32;
        fn to_array(&self) -> [u32; 3] {
            self.to_array()
        }
    }
    impl Point for U64Vec3 {
        type Data = u64;
        fn to_array(&self) -> [u64; 3] {
            self.to_array()
        }
    }

    impl Point for I16Vec3 {
        type Data = i16;
        fn to_array(&self) -> [i16; 3] {
            self.to_array()
        }
    }
    impl Point for IVec3 {
        type Data = i32;
        fn to_array(&self) -> [i32; 3] {
            self.to_array()
        }
    }
    impl Point for I64Vec3 {
        type Data = i64;
        fn to_array(&self) -> [i64; 3] {
            self.to_array()
        }
    }
}
