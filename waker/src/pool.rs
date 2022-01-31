use ach_array::Array;
use core::task::Waker;

/// The token is ownership of a Cell of the pool.
///
/// The waker will drop without wake, if the token drop
pub struct WakerToken<'a, const N: usize> {
    pool: &'a WakerPool<N>,
    index: usize,
}
impl<'a, const N: usize> WakerToken<'a, N> {
    /// Replace waker
    ///
    /// wake it immediately, if is waking.
    pub fn replace(&self, waker: &Waker) {
        if let Err(_) = self.pool.pool[self.index].try_replace(waker.clone()) {
            waker.wake_by_ref();
            return;
        }
    }
}
impl<'a, const N: usize> Drop for WakerToken<'a, N> {
    fn drop(&mut self) {
        if let Ok(wake) = self.pool.pool[self.index].try_get() {
            wake.remove();
        }
    }
}

pub struct WakerPool<const N: usize> {
    pool: Array<Waker, N>,
}
impl<const N: usize> WakerPool<N> {
    pub const fn new() -> Self {
        Self { pool: Array::new() }
    }
    /// Hold a place in the pool
    ///
    /// Returns None if the pool is full.
    pub fn register(&self, waker: &Waker) -> Option<WakerToken<N>> {
        if let Ok(index) = self.pool.push(waker.clone()) {
            Some(WakerToken { pool: self, index })
        } else {
            None
        }
    }
    /// Only wake all, not remove it
    pub fn wake(&self) {
        for waker in self.pool.iter(false) {
            waker.wake_by_ref();
        }
    }
}
