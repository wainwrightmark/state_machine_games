use std::{
    iter::FusedIterator,
    sync::atomic::{AtomicU32, Ordering},
};

use const_sized_bit_set::BitSet32;

/// A Bag with at most 32 elements
#[derive(Debug)]
pub struct LittleBag<T> {
    elements: Vec<T>,
    live_elements_set: AtomicU32,
}

impl<T> LittleBag<T> {
    pub const fn new() -> Self {
        Self {
            elements: vec![],
            live_elements_set: AtomicU32::new(0),
        }
    }

    pub fn new_from_vec(elements: Vec<T>) -> Self {
        let live_elements_set =
            AtomicU32::new(BitSet32::from_first_n_const(elements.len() as u32).inner_const());

        Self {
            elements,
            live_elements_set,
        }
    }

    pub fn is_empty(&self)-> bool{
        self.live_elements_set.load(Ordering::Acquire).count_ones() == 0
    }

    pub fn len(&self) -> usize {
        self.live_elements_set.load(Ordering::Acquire).count_ones() as usize
    }

    ///Does not require an exclusive reference
    pub fn clear(&self) {
        self.live_elements_set.store(0, Ordering::Release);
    }

    // pub fn clear_then_populate(&mut self, iter: impl Iterator<Item = T>) {
    //     self.elements.clear();
    //     self.elements.extend(iter);
    //     self.live_elements_set =
    //         AtomicU32::new(BitSet32::from_first_n_const(self.elements.len() as u32).inner_const());
    // }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &T> + ExactSizeIterator + DoubleEndedIterator + FusedIterator {
        let set = BitSet32::from_inner_const(self.live_elements_set.load(Ordering::Acquire));

        set.into_iter()
            .map(|index| self.elements.get(index as usize).unwrap())
    }

    ///Note that this does not require an exclusive reference to the bag
    pub fn retain(&self, mut f: impl FnMut(&T) -> bool) {
        let mut set = BitSet32::from_inner_const(self.live_elements_set.load(Ordering::Acquire));

        let iter = set.into_iter();

        for index in iter {
            let v = self.elements.get(index as usize).unwrap();
            if !f(&v) {
                set.remove_const(index);
            }
        }
        self.live_elements_set
            .store(set.inner_const(), std::sync::atomic::Ordering::Release);
    }
}
