#![no_std]

use ach_util::Error;
use async_ach_ring::Ring;
use core::ops::Deref;

pub struct Sender<'a, T, const N: usize> {
    mpmc: &'a Mpmc<T, N>,
}
impl<'a, T, const N: usize> Sender<'a, T, N> {
    const fn new(mpmc: &'a Mpmc<T, N>) -> Self {
        Sender { mpmc }
    }
}
impl<'a, T: Unpin, const N: usize> Sender<'a, T, N> {
    /// Appends an element to the back of the Ring.
    ///
    /// Returns Err if the Ring is full or in critical section.
    pub fn try_send(&self, t: T) -> Result<(), Error<T>> {
        self.mpmc.try_push(t)
    }
    /// Appends an element to the back of the Ring.
    pub async fn send(&self, t: T) {
        self.mpmc.push(t).await
    }
}

pub struct Receiver<'a, T, const N: usize> {
    mpmc: &'a Mpmc<T, N>,
}
impl<'a, T, const N: usize> Receiver<'a, T, N> {
    const fn new(mpmc: &'a Mpmc<T, N>) -> Self {
        Receiver { mpmc }
    }
}
impl<'a, T: Unpin, const N: usize> Receiver<'a, T, N> {
    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty or in critical section.
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.mpmc.try_pop()
    }
    /// Removes the first element and returns it.
    pub async fn recv(&self) -> T {
        self.mpmc.pop().await
    }
}

pub struct Mpmc<T, const N: usize> {
    ring: Ring<T, N>,
}
impl<T, const N: usize> Mpmc<T, N> {
    pub const fn new() -> Self {
        Self { ring: Ring::new() }
    }
    pub const fn sender(&self) -> Sender<T, N> {
        Sender::new(self)
    }
    pub const fn recver(&self) -> Receiver<T, N> {
        Receiver::new(self)
    }
}
impl<T, const N: usize> Deref for Mpmc<T, N> {
    type Target = Ring<T, N>;
    fn deref(&self) -> &Self::Target {
        &self.ring
    }
}
