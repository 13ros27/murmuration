use std::{
    array,
    iter::{Enumerate, Map, Zip},
    slice,
};

pub trait ConstIterator<const N: usize> {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstWrapper<I: Iterator, const N: usize>(I);

impl<I: Iterator, const N: usize> Iterator for ConstWrapper<I, N> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        self.0.next()
    }
}

impl<I: Iterator, const N: usize> ConstIterator<N> for ConstWrapper<I, N> {}

pub trait ConstArrayIter<'a, const N: usize> {
    type Iter: ConstIterator<N>;

    fn const_iter(&'a self) -> Self::Iter;
}

impl<'a, T: 'a, const N: usize> ConstArrayIter<'a, N> for [T; N] {
    type Iter = ConstWrapper<slice::Iter<'a, T>, N>;

    fn const_iter(&'a self) -> Self::Iter {
        ConstWrapper(self.iter())
    }
}

pub trait ConstArrayIterMut<'a, const N: usize> {
    type Iter: ConstIterator<N>;

    fn const_iter_mut(&'a mut self) -> Self::Iter;
}

impl<'a, T: 'a, const N: usize> ConstArrayIterMut<'a, N> for [T; N] {
    type Iter = ConstWrapper<slice::IterMut<'a, T>, N>;

    fn const_iter_mut(&'a mut self) -> Self::Iter {
        ConstWrapper(self.iter_mut())
    }
}

impl<T, const N: usize> ConstIterator<N> for array::IntoIter<T, N> {}

impl<I: ConstIterator<N>, const N: usize> ConstIterator<N> for Enumerate<I> {}

impl<I: ConstIterator<N>, F, const N: usize> ConstIterator<N> for Map<I, F> {}

impl<A: ConstIterator<N>, B: ConstIterator<N>, const N: usize> ConstIterator<N> for Zip<A, B> {}

pub trait ConstCollect<const N: usize> {
    type Collected;

    fn const_collect(self) -> Self::Collected;
}

impl<I: ConstIterator<N> + Iterator<Item = T>, T, const N: usize> ConstCollect<N> for I {
    type Collected = [T; N];

    fn const_collect(self) -> [T; N] {
        let collected = self.collect::<Vec<T>>().try_into();
        // SAFETY: ConstIterator<N> checks it was always the correct size
        unsafe { collected.unwrap_unchecked() }
    }
}
