use super::unsigned::Unsigned;
use std::ops::{Add, Mul, Sub};

pub trait OrderedBinary:
    Clone + PartialEq + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
    const ZERO: Self;
    type Ordered: Unsigned;
    fn to_ordered(&self) -> Self::Ordered;
    fn from_ordered(ordered: Self::Ordered) -> Self;

    /// This should be overridden for unsigned types
    fn distance_squared(&self, other: &Self) -> Self {
        let dist = self.clone() - other.clone();
        dist.clone() * dist
    }

    /// Used to filter out NaNs from floats, should probably find a better solution
    fn is_irrelevant(&self) -> bool {
        false
    }
}

impl<U: Unsigned> OrderedBinary for U {
    const ZERO: Self = Self::ZERO;
    type Ordered = Self;
    fn to_ordered(&self) -> Self {
        *self
    }
    fn from_ordered(ordered: Self) -> Self {
        ordered
    }
    fn distance_squared(&self, other: &Self) -> Self {
        let dist = if self > other {
            *self - *other
        } else {
            *other - *self
        };
        dist * dist
    }
}

impl OrderedBinary for i16 {
    const ZERO: i16 = 0;
    type Ordered = u16;
    fn to_ordered(&self) -> Self::Ordered {
        u16::from_ne_bytes(self.to_ne_bytes()) ^ (1_u16 << 15)
    }
    fn from_ordered(ordered: u16) -> Self {
        i16::from_ne_bytes((ordered ^ (1_u16 << 15)).to_ne_bytes())
    }
}

impl OrderedBinary for i32 {
    const ZERO: i32 = 0;
    type Ordered = u32;
    fn to_ordered(&self) -> Self::Ordered {
        u32::from_ne_bytes(self.to_ne_bytes()) ^ (1_u32 << 31)
    }
    fn from_ordered(ordered: u32) -> Self {
        i32::from_ne_bytes((ordered ^ (1_u32 << 31)).to_ne_bytes())
    }
}

impl OrderedBinary for i64 {
    const ZERO: i64 = 0;
    type Ordered = u64;
    fn to_ordered(&self) -> Self::Ordered {
        u64::from_ne_bytes(self.to_ne_bytes()) ^ (1_u64 << 63)
    }
    fn from_ordered(ordered: u64) -> Self {
        i64::from_ne_bytes((ordered ^ (1_u64 << 63)).to_ne_bytes())
    }
}

impl OrderedBinary for f32 {
    const ZERO: f32 = 0.0;
    type Ordered = u32;
    fn to_ordered(&self) -> Self::Ordered {
        u32::from_ne_bytes(self.to_ne_bytes()) ^ (1_u32 << 31)
    }
    fn from_ordered(ordered: u32) -> Self {
        f32::from_ne_bytes((ordered ^ (1_u32 << 31)).to_ne_bytes())
    }
    fn is_irrelevant(&self) -> bool {
        self.is_nan()
    }
}

impl OrderedBinary for f64 {
    const ZERO: f64 = 0.0;
    type Ordered = u64;
    fn to_ordered(&self) -> Self::Ordered {
        u64::from_ne_bytes(self.to_ne_bytes()) ^ (1_u64 << 63)
    }
    fn from_ordered(ordered: u64) -> Self {
        f64::from_ne_bytes((ordered ^ (1_u64 << 63)).to_ne_bytes())
    }
    fn is_irrelevant(&self) -> bool {
        self.is_nan()
    }
}
