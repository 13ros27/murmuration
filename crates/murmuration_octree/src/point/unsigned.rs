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
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + From<u8>
    + TryInto<u8>
    + Copy
    + Debug
    + Eq
    + Ord
    + Hash
    + Send
    + Sync
{
    const ZERO: Self;
    const MAX: Self;
    fn leading_zeros(self) -> u8;
    fn overflowing_shl(self, rhs: u32) -> Self;
    fn overflowing_shr(self, rhs: u32) -> Self;

    fn clear_lower(self, n: u8) -> Self {
        if n == 32 {
            Self::ZERO
        } else {
            self >> n << n
        }
    }
}

impl Unsigned for u8 {
    const ZERO: u8 = 0;
    const MAX: u8 = u8::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
    fn overflowing_shl(self, rhs: u32) -> Self {
        self.overflowing_shl(rhs).0
    }
    fn overflowing_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }
}
impl Unsigned for u16 {
    const ZERO: u16 = 0;
    const MAX: u16 = u16::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
    fn overflowing_shl(self, rhs: u32) -> Self {
        self.overflowing_shl(rhs).0
    }
    fn overflowing_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }
}
impl Unsigned for u32 {
    const ZERO: u32 = 0;
    const MAX: u32 = u32::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
    fn overflowing_shl(self, rhs: u32) -> Self {
        self.overflowing_shl(rhs).0
    }
    fn overflowing_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }
}
impl Unsigned for u64 {
    const ZERO: u64 = 0;
    const MAX: u64 = u64::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
    fn overflowing_shl(self, rhs: u32) -> Self {
        self.overflowing_shl(rhs).0
    }
    fn overflowing_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }
}
impl Unsigned for u128 {
    const ZERO: u128 = 0;
    const MAX: u128 = u128::MAX;
    fn leading_zeros(self) -> u8 {
        self.leading_zeros() as u8
    }
    fn overflowing_shl(self, rhs: u32) -> Self {
        self.overflowing_shl(rhs).0
    }
    fn overflowing_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }
}
