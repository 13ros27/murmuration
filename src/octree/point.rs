use bytemuck::Pod;
use glam::{IVec3, U64Vec3, UVec3};
use std::cmp::Ordering;
use std::fmt::{Binary, Debug};
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
    + Binary
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
    const ZERO: Self;
    const MAX: Self;
    fn leading_zeros(self) -> u8;
}

impl Unsigned for u8 {
    const ZERO: u8 = 0;
    const MAX: u8 = u8::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u16 {
    const ZERO: u16 = 0;
    const MAX: u16 = u16::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u32 {
    const ZERO: u32 = 0;
    const MAX: u32 = u32::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u64 {
    const ZERO: u64 = 0;
    const MAX: u64 = u64::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}
impl Unsigned for u128 {
    const ZERO: u128 = 0;
    const MAX: u128 = u128::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
}

pub trait OrderedBinary:
    Clone + PartialEq + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
    const ZERO: Self;
    type Ordered: Unsigned;
    fn to_ordered(&self) -> Self::Ordered;
    fn from_ordered(ordered: Self::Ordered) -> Self;
    fn distance_squared(&self, other: &Self) -> Self {
        let dist = if self > other {
            self.clone() - other.clone()
        } else {
            other.clone() - self.clone()
        };
        dist.clone() * dist
    }
}

impl OrderedBinary for u32 {
    const ZERO: u32 = 0;
    type Ordered = u32;
    fn to_ordered(&self) -> u32 {
        *self
    }
    fn from_ordered(ordered: u32) -> Self {
        ordered
    }
}

impl OrderedBinary for u64 {
    const ZERO: u64 = 0;
    type Ordered = u64;
    fn to_ordered(&self) -> u64 {
        *self
    }
    fn from_ordered(ordered: u64) -> Self {
        ordered
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
    fn distance_squared(&self, other: &Self) -> Self {
        let dist = self.clone() - other.clone();
        dist.clone() * dist
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
    pub(crate) const ZERO: Self = Self([<P::Data as OrderedBinary>::Ordered::ZERO; 3]);

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

    pub(crate) fn distance_squared(&self, other: &Self) -> P::Data {
        let (self_x, other_x, self_y, other_y, self_z, other_z) = (
            P::Data::from_ordered(self.0[0]),
            P::Data::from_ordered(other.0[0]),
            P::Data::from_ordered(self.0[1]),
            P::Data::from_ordered(other.0[1]),
            P::Data::from_ordered(self.0[2]),
            P::Data::from_ordered(other.0[2]),
        );
        self_x.distance_squared(&other_x)
            + self_y.distance_squared(&other_y)
            + self_z.distance_squared(&other_z)
    }

    /// Returns the squared distance between self and centre (if we go down the ind branch) as a best case at this depth
    pub(crate) fn closest_distance(&self, ind: u8, centre: &Self, depth: u8) -> P::Data {
        // We use self from above depth - 1, ind at depth, and then compare with centre
        let shift = P::MAX_DEPTH - depth;
        let increase_mask = <P::Data as OrderedBinary>::Ordered::MAX >> depth;

        let mut dist = P::Data::ZERO;
        for i in 0..=2 {
            let ind_part: <P::Data as OrderedBinary>::Ordered = (ind & (4 >> i)).into();
            let child = if shift >= 31 {
                0.into()
            } else {
                self.0[i] >> (shift + 1) << (shift + 1)
            } | (ind_part >> (2 - i as u8) << shift);
            let centre_data = P::Data::from_ordered(centre.0[i]);
            let centre = centre.0[i] >> shift << shift;
            dist = dist
                + match child.cmp(&centre) {
                    Ordering::Equal => P::Data::ZERO,
                    Ordering::Less => {
                        centre_data.distance_squared(&P::Data::from_ordered(child | increase_mask))
                    }
                    Ordering::Greater => {
                        centre_data.distance_squared(&P::Data::from_ordered(child))
                    }
                };
        }
        dist
    }

    /// Combine an index from .nth with self at the given depth
    pub(crate) fn combine_ind(&self, ind: u8, depth: u8) -> Self {
        let shift = P::MAX_DEPTH - depth + 1;
        let ind_4: <P::Data as OrderedBinary>::Ordered = (ind & 4).into();
        let ind_2: <P::Data as OrderedBinary>::Ordered = (ind & 2).into();
        let ind_1: <P::Data as OrderedBinary>::Ordered = (ind & 1).into();
        if shift >= 32 {
            PointData([
                ind_4 >> (shift - 3),
                ind_2 >> (shift - 2),
                ind_1 >> (shift - 1),
            ])
        } else {
            PointData([
                self.0[0] >> shift << shift | ind_4 >> 3 << shift,
                self.0[1] >> shift << shift | ind_2 >> 2 << shift,
                self.0[2] >> shift << shift | ind_1 >> 1 << shift,
            ])
        }
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
    fn to_array(&self) -> [Self::Data; 3];
    fn get_point(&self) -> PointData<Self> {
        let arr = self.to_array();
        PointData([
            arr[0].to_ordered(),
            arr[1].to_ordered(),
            arr[2].to_ordered(),
        ])
    }

    const MAX_DEPTH: u8 = (std::mem::size_of::<Self::Data>() * 8) as u8;
}

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
impl Point for U64Vec3 {
    type Data = u64;
    fn to_array(&self) -> [u64; 3] {
        self.to_array()
    }
}
