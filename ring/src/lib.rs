#![no_std]

use ach_ring as ach;
use ach_util::Error;
use async_ach_notify::Notify;

pub struct Ring<T, const N: usize> {
    buf: ach::Ring<T, N>,
    consumer: Notify,
    producer: Notify,
}
impl<T, const N: usize> Ring<T, N> {
    pub const fn new() -> Self {
        Self {
            buf: ach::Ring::new(),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
}
impl<T: Unpin, const N: usize> Ring<T, N> {
    /// Appends an element to the back of the Ring.
    ///
    /// Returns Err if the Ring is full or in critical section.
    pub fn try_push(&self, val: T) -> Result<(), Error<T>> {
        self.buf.try_push(val).map(|x| {
            self.producer.notify_one();
            x
        })
    }
    /// Appends an element to the back of the Ring.
    pub async fn push(&self, mut val: T) {
        loop {
            if let Err(err) = self.try_push(val) {
                val = err.input;
                self.consumer.listen().await;
            } else {
                break;
            }
        }
    }

    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty or in critical section.
    pub fn try_pop(&self) -> Result<T, Error<()>> {
        self.buf.try_pop().map(|x| {
            self.consumer.notify_one();
            x
        })
    }
    /// Removes the first element and returns it.
    pub async fn pop(&self) -> T {
        loop {
            if let Ok(v) = self.try_pop() {
                break v;
            } else {
                self.producer.listen().await;
            }
        }
    }
}
