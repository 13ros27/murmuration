pub mod ordered;
pub mod unsigned;

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::ops::BitXor;

use ordered::OrderedBinary;
use unsigned::Unsigned;

/// The underlying ordered type used for positioning in the [`Octree`](crate::Octree).
#[derive(Clone, Eq, PartialOrd, Ord)]
pub struct PointData<P: Point>(pub [<P::Data as OrderedBinary>::Ordered; 3]);

impl<P: Point> PartialEq for PointData<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P: Point> Debug for PointData<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.map(|n| format!("{n:0>0$b}", P::MAX_DEPTH as usize)))
            .finish()
    }
}

impl<P: Point> PointData<P> {
    pub(crate) const ZERO: Self =
        Self([<<P::Data as OrderedBinary>::Ordered as OrderedBinary>::ZERO; 3]);

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
            } | (ind_part >> (2 - i as u8).overflowing_shl(shift as u32).0);
            let centre_data = P::Data::from_ordered(centre.0[i]);
            let centre = centre.0[i]
                .overflowing_shr(shift as u32)
                .overflowing_shl(shift as u32);
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

    pub(crate) fn closest_distance_new(&self, centre: &Self, depth: u8) -> P::Data {
        let shift = P::MAX_DEPTH - depth - 1;
        let increase_mask = <P::Data as OrderedBinary>::Ordered::MAX.overflowing_shr(depth as u32);

        let mut dist = P::Data::ZERO;
        for i in 0..=2 {
            let centre_data = P::Data::from_ordered(centre.0[i]);
            let centre_cut = centre.0[i] >> shift << shift;
            let child_cut = self.0[i] >> shift << shift;

            dist = dist
                + match child_cut.cmp(&centre_cut) {
                    Ordering::Equal => P::Data::ZERO,
                    Ordering::Less => centre_data
                        .distance_squared(&P::Data::from_ordered(child_cut | increase_mask)),
                    Ordering::Greater => {
                        centre_data.distance_squared(&P::Data::from_ordered(child_cut))
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
                ind_4 << (shift - 3),
                ind_2 << (shift - 2),
                ind_1 << (shift - 1),
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

/// A type which can be used as the coordinate system in an [`Octree`](crate::Octree).
pub trait Point: Clone + Sized {
    /// The underlying coordinate number type it uses.
    type Data: OrderedBinary;
    /// Returns the data stored in this point.
    fn to_array(&self) -> [Self::Data; 3];
    /// Converts the `Point` type into a `PointData` so that it can be used to index the [`Octree`](crate::Octree).
    fn get_point(&self) -> PointData<Self> {
        let arr = self.to_array();
        PointData([
            arr[0].to_ordered(),
            arr[1].to_ordered(),
            arr[2].to_ordered(),
        ])
    }

    /// The number of bits stored in this point's `Self::Data`.
    const MAX_DEPTH: u8 = (std::mem::size_of::<Self::Data>() * 8) as u8;
}
