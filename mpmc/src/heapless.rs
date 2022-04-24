use ach_util::Error;
use async_ach_ring::Ring;
use core::ops::Deref;

pub struct Sender<'a, T, const N: usize, const MP: usize, const MC: usize> {
    mpmc: &'a Mpmc<T, N, MP, MC>,
}
impl<'a, T, const N: usize, const MP: usize, const MC: usize> Sender<'a, T, N, MP, MC> {
    const fn new(mpmc: &'a Mpmc<T, N, MP, MC>) -> Self {
        Sender { mpmc }
    }
}
impl<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> Sender<'a, T, N, MP, MC> {
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

pub struct Receiver<'a, T, const N: usize, const MP: usize, const MC: usize> {
    mpmc: &'a Mpmc<T, N, MP, MC>,
}
impl<'a, T, const N: usize, const MP: usize, const MC: usize> Receiver<'a, T, N, MP, MC> {
    const fn new(mpmc: &'a Mpmc<T, N, MP, MC>) -> Self {
        Receiver { mpmc }
    }
}
impl<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> Receiver<'a, T, N, MP, MC> {
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

pub struct Mpmc<T, const N: usize, const MP: usize, const MC: usize> {
    ring: Ring<T, N, MP, MC>,
}
impl<T, const N: usize, const MP: usize, const MC: usize> Mpmc<T, N, MP, MC> {
    pub const fn new() -> Self {
        Self { ring: Ring::new() }
    }
    pub const fn sender(&self) -> Sender<T, N, MP, MC> {
        Sender::new(self)
    }
    pub const fn recver(&self) -> Receiver<T, N, MP, MC> {
        Receiver::new(self)
    }
}
impl<T, const N: usize, const MP: usize, const MC: usize> Deref for Mpmc<T, N, MP, MC> {
    type Target = Ring<T, N, MP, MC>;
    fn deref(&self) -> &Self::Target {
        &self.ring
    }
}
