use bytemuck::Pod;
use glam::{U64Vec3, UVec3};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Shl, Shr, Sub};

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for u128 {}
}

pub trait Unsigned:
    sealed::Sealed
    + BitAnd<Self, Output = Self>
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
    const MAX: Self;
    fn leading_zeros(self) -> u8;
}

impl Unsigned for u8 {
    const MAX: u8 = u8::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u16 {
    const MAX: u16 = u16::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u32 {
    const MAX: u32 = u32::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u64 {
    const MAX: u64 = u64::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u128 {
    const MAX: u128 = u128::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}

pub trait OrderedBinary:
    Clone + PartialEq + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
    type Ordered: Unsigned;
    fn to_ordered(self) -> Self::Ordered;
    fn from_ordered(ordered: Self::Ordered) -> Self;
}

impl OrderedBinary for u32 {
    type Ordered = u32;
    fn to_ordered(self) -> u32 {
        self
    }
    fn from_ordered(ordered: u32) -> Self {
        ordered
    }
}

impl OrderedBinary for u64 {
    type Ordered = u64;
    fn to_ordered(self) -> u64 {
        self
    }
    fn from_ordered(ordered: u64) -> Self {
        ordered
    }
}

#[derive(Clone, Eq, PartialOrd, Ord, Debug)]
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

    pub(crate) fn cross_or(&self) -> <P::Data as OrderedBinary>::Ordered {
        self.0[0] | self.0[1] | self.0[2]
    }

    /// Returns the smallest number of leading zeros from all of its contained numbers
    pub(crate) fn leading_zeros(&self) -> u8 {
        self.cross_or().leading_zeros()
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

    pub(crate) fn approximate_closest(&self, centre: &Self, depth: u8) -> Self {
        let shift = P::MAX_DEPTH - depth;
        // Sadly zip seems to (at least sometimes) kill performance so manual it is
        let x = if (centre.0[0] >> shift & 1.into()) == 1.into() {
            self.0[0] | (<P::Data as OrderedBinary>::Ordered::MAX >> depth)
        } else {
            self.0[0] & (<P::Data as OrderedBinary>::Ordered::MAX << (32 - depth))
        };
        let y = if (centre.0[1] >> shift & 1.into()) == 1.into() {
            self.0[1] | (<P::Data as OrderedBinary>::Ordered::MAX >> depth)
        } else {
            self.0[1] & (<P::Data as OrderedBinary>::Ordered::MAX << (32 - depth))
        };
        let z = if (centre.0[2] >> shift & 1.into()) == 1.into() {
            self.0[2] | (<P::Data as OrderedBinary>::Ordered::MAX >> depth)
        } else {
            self.0[2] & (<P::Data as OrderedBinary>::Ordered::MAX << (32 - depth))
        };
        PointData([x, y, z])
    }

    pub(crate) fn distance_squared(&self, other: &Self) -> P::Data {
        // TODO: Maybe separately handle those that are fine with negatives?
        let (self_x, other_x, self_y, other_y, self_z, other_z) = (
            P::Data::from_ordered(self.0[0]),
            P::Data::from_ordered(other.0[0]),
            P::Data::from_ordered(self.0[1]),
            P::Data::from_ordered(other.0[1]),
            P::Data::from_ordered(self.0[2]),
            P::Data::from_ordered(other.0[2]),
        );
        let dist_x = if self_x > other_x {
            self_x - other_x
        } else {
            other_x - self_x
        };
        let dist_y = if self_y > other_y {
            self_y - other_y
        } else {
            other_y - self_y
        };
        let dist_z = if self_z > other_z {
            self_z - other_z
        } else {
            other_z - self_z
        };
        dist_x.clone() * dist_x + dist_y.clone() * dist_y + dist_z.clone() * dist_z
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
        PointData(self.to_array())
    }
}

impl Point for U64Vec3 {
    type Data = u64;
    fn get_point(&self) -> PointData<Self> {
        PointData(self.to_array())
    }
}
