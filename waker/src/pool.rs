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
    /// Swap waker
    ///
    /// wake it immediately, if is waking.
    ///
    /// Notice: `Spin`
    pub fn swap(&self, waker: &Waker) {
        if self.pool.pool.swap(self.index, waker.clone()).is_err() {
            waker.wake_by_ref();
        }
    }
}
impl<'a, const N: usize> Drop for WakerToken<'a, N> {
    fn drop(&mut self) {
        if let Some(wake) = self.pool.pool.get(self.index) {
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
    pub fn register(&self, waker: &Waker) -> Result<WakerToken<N>, ()> {
        if let Ok(index) = self.pool.push(waker.clone()) {
            Ok(WakerToken { pool: self, index })
        } else {
            Err(())
        }
    }
    /// Only wake all, not remove it
    ///
    /// Notice: `Spin`
    pub fn wake(&self) {
        for w in self.pool.iter(true) {
            let waker: Waker = w.clone();
            drop(w);
            waker.wake();
        }
    }
}
