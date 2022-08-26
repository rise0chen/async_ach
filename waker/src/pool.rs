use crate::WakerEntity;
use ach_array::Array;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct WakerToken<'a, T, const N: usize> {
    pool: &'a WakerPool<T, N>,
    index: usize,
}
impl<'a, T, const N: usize> WakerToken<'a, T, N> {
    /// Swap waker
    ///
    /// wake it immediately, if is waking.
    pub fn swap(&self, waker: WakerEntity<T>) {
        if let Err(e) = self.pool.pool[self.index].try_replace(waker) {
            e.input.wake();
        }
    }
}
impl<'a, T, const N: usize> Drop for WakerToken<'a, T, N> {
    fn drop(&mut self) {
        if let Ok(wake) = self.pool.pool[self.index].get() {
            wake.remove();
        }
        let _ = self.pool.used[self.index].compare_exchange(
            true,
            false,
            Ordering::SeqCst,
            Ordering::Relaxed,
        );
    }
}

pub struct WakerPool<T, const N: usize> {
    pool: Array<WakerEntity<T>, N>,
    used: [AtomicBool; N],
}
impl<T, const N: usize> WakerPool<T, N> {
    const FALSE: AtomicBool = AtomicBool::new(false);
    pub const fn new() -> Self {
        Self {
            pool: Array::new(),
            used: [Self::FALSE; N],
        }
    }
    /// Hold a place in the pool
    pub fn register(&self) -> Result<WakerToken<T, N>, ()> {
        for (i, used) in self.used.iter().enumerate() {
            if used
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(WakerToken {
                    pool: self,
                    index: i,
                });
            }
        }
        Err(())
    }
    /// Wake a waiter, and remove it.
    ///
    /// Returns false if the pool is empty.
    pub fn wake_one(&self) -> bool {
        loop {
            if let Some(waker) = self.pool.pop() {
                if waker.wake() {
                    return true;
                }
            } else {
                return false;
            }
        }
    }
    /// Wake all waiter, and remove it.
    ///
    /// returns the number of had waked
    pub fn wake_all(&self) -> usize {
        let mut num = 0;
        while let Some(waker) = self.pool.pop() {
            waker.wake();
            num += 1;
        }
        num
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that f(&e) returns false.
    /// This method operates in place, visiting each element exactly once in the original order,
    /// but not preserves the order of the retained elements.
    pub fn retain(&self, mut f: impl FnMut(&WakerEntity<T>) -> bool) {
        for e in self.pool.iter(false) {
            if !f(&*e) {
                // Remove it
                e.remove();
            }
        }
    }
}
