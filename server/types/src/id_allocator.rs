use hibitset::{AtomicBitSet, DrainableBitSet};
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};

#[derive(Default)]
pub struct IdCache {
    cache: Vec<u16>,
    length: AtomicUsize,
}

impl IdCache {
    fn maintain<I>(&mut self, iter: I)
    where
        I: Iterator<Item = u16>,
    {
        let len = self.length.get_mut();
        self.cache.truncate(*len);
        self.cache.extend(iter);
        *len = self.cache.len();
    }

    fn pop(&self) -> Option<u16> {
        let mut idx = self.length.load(Ordering::Relaxed);
        if idx == 0 {
            return None;
        }
        loop {
            let old_idx = self
                .length
                .compare_and_swap(idx, idx - 1, Ordering::Relaxed);
            if old_idx == 0 {
                return None;
            } else if old_idx == idx {
                return Some(self.cache[idx]);
            }
            idx = old_idx;
        }
    }
}

#[derive(Default)]
pub struct IdAllocator {
    cache: IdCache,
    next: AtomicU16,
    released: AtomicBitSet,
}

impl IdAllocator {
    pub fn acquire(&self) -> u16 {
        self.cache.pop().unwrap_or_else(|| {
            let id = self.next.fetch_add(1, Ordering::Relaxed);
            if id == u16::MAX {
                panic!("We ran out of IDs D:")
            }
            id
        })
    }

    pub fn release(&self, id: u16) {
        self.released.add_atomic(id.into());
    }

    pub fn maintain(&mut self) {
        self.cache
            .maintain(self.released.drain().map(|id| id as u16));
    }
}
