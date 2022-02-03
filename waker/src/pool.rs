use ach_array::Array;
use core::task::Waker;

pub struct WakerPool<const N: usize> {
    pool: Array<Waker, N>,
}
impl<const N: usize> WakerPool<N> {
    pub const fn new() -> Self {
        Self { pool: Array::new() }
    }
    /// register a waker into the pool
    ///
    /// Returns false if the pool is full.
    #[must_use]
    pub fn register(&self, waker: &Waker) -> bool {
        if let Ok(_) = self.pool.push(waker.clone()) {
            true
        } else {
            false
        }
    }
    /// Wake a waiter, and remove it.
    ///
    /// Returns false if the pool is empty.
    pub fn wake_one(&self) -> bool {
        if let Some(waker) = self.pool.pop() {
            waker.wake();
            true
        } else {
            false
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
}
