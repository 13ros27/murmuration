use glam::{U64Vec3, UVec3};
use std::ops::BitXor;

use sealed::Unsigned;

mod sealed {
    use bytemuck::Pod;
    use std::{
        fmt::Debug,
        hash::Hash,
        ops::{BitAnd, BitOr, BitXor, Shl, Shr},
    };

    pub trait Unsigned:
        BitAnd<Self, Output = Self>
        + BitOr<Output = Self>
        + BitXor<Output = Self>
        + Shl<u8, Output = Self>
        + Shr<u8, Output = Self>
        + From<u8>
        + TryInto<u8>
        + Copy
        + Debug
        + Eq
        + Ord
        + Hash
        + Pod
        + Send
        + Sync
    {
        fn leading_zeros(self) -> u8;
    }

    impl Unsigned for u8 {
        fn leading_zeros(self) -> u8 {
            self.leading_zeros() as u8
        }
    }
    impl Unsigned for u16 {
        fn leading_zeros(self) -> u8 {
            self.leading_zeros() as u8
        }
    }
    impl Unsigned for u32 {
        fn leading_zeros(self) -> u8 {
            self.leading_zeros() as u8
        }
    }
    impl Unsigned for u64 {
        fn leading_zeros(self) -> u8 {
            self.leading_zeros() as u8
        }
    }
    impl Unsigned for u128 {
        fn leading_zeros(self) -> u8 {
            self.leading_zeros() as u8
        }
    }
}

pub trait OrderedBinary: Clone + PartialEq {
    type Ordered: sealed::Unsigned;
    fn to_ordered(self) -> Self::Ordered;
}

impl OrderedBinary for u32 {
    type Ordered = u32;
    fn to_ordered(self) -> u32 {
        self
    }
}

impl OrderedBinary for u64 {
    type Ordered = u64;
    fn to_ordered(self) -> u64 {
        self
    }
}

#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct PointData<P: Point>([<P::Data as OrderedBinary>::Ordered; 3]);

impl<P: Point> PartialEq for PointData<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P: Point> PointData<P> {
    pub fn new(data: [<P::Data as OrderedBinary>::Ordered; 3]) -> Self {
        Self(data)
    }

    /// Returns the smallest number of leading zeros from all of its contained numbers
    pub(crate) fn leading_zeros(&self) -> u8 {
        let min = self.0.iter().map(|b| b.leading_zeros()).min();
        // SAFETY: The length of self.0 must be 3 so it must always have a minimum
        unsafe { min.unwrap_unchecked() }
    }

    /// Get the 3 values at n as a binary number (essentially a binary cross-section)
    pub(crate) fn nth(&self, n: u8) -> u8 {
        let shift = P::MAX_DEPTH - 1 - n;
        let val = (self.0[0] >> shift & 1.into()) << 2
            | (self.0[1] >> shift & 1.into()) << 1
            | (self.0[2] >> shift & 1.into());
        // SAFETY: It is safe to cast into a u8 because it can be at most 7
        unsafe { val.try_into().unwrap_unchecked() }
    }
}

impl<P: Point> BitXor for &PointData<P> {
    type Output = PointData<P>;
    fn bitxor(self, rhs: Self) -> PointData<P> {
        PointData([
            self.0[0] ^ rhs.0[0],
            self.0[1] ^ rhs.0[1],
            self.0[2] ^ rhs.0[2],
        ])
    }
}

pub trait Point: Clone + Sized {
    type Data: OrderedBinary;
    fn get_point(&self) -> PointData<Self>;

    const MAX_DEPTH: u8 = (std::mem::size_of::<Self::Data>() * 8) as u8;
}

impl Point for UVec3 {
    type Data = u32;
    fn get_point(&self) -> PointData<Self> {
        PointData([self.x, self.y, self.z])
    }
}

impl Point for U64Vec3 {
    type Data = u64;
    fn get_point(&self) -> PointData<Self> {
        PointData([self.x, self.y, self.z])
    }
}
