#![no_std]

use ach_ring as ach;
use ach_util::Error;
use async_ach_notify::Notify;
use futures_util::StreamExt;

pub struct Ring<T, const N: usize, const MP: usize, const MC: usize> {
    buf: ach::Ring<T, N>,
    consumer: Notify<MP>,
    producer: Notify<MC>,
}
impl<T, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
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
impl<T: Unpin, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
    /// Appends an element to the back of the Ring.
    ///
    /// Returns Err if the Ring is full or in critical section.
    pub fn try_push(&self, val: T) -> Result<(), Error<T>> {
        self.buf.push(val).map(|x| {
            self.producer.notify_one();
            x
        })
    }
    /// Appends an element to the back of the Ring.
    pub async fn push(&self, mut val: T) {
        let mut wait_c = self.consumer.listen();
        loop {
            if let Err(err) = self.try_push(val) {
                val = err.input;
                wait_c.next().await;
            } else {
                break;
            }
        }
    }

    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty or in critical section.
    pub fn try_pop(&self) -> Result<T, Error<()>> {
        self.buf.pop().map(|x| {
            self.consumer.notify_one();
            x
        })
    }
    /// Removes the first element and returns it.
    pub async fn pop(&self) -> T {
        let mut wait_p = self.producer.listen();
        loop {
            if let Ok(v) = self.try_pop() {
                break v;
            } else {
                wait_p.next().await;
            }
        }
    }
}
