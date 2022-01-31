#![no_std]

use ach_cell::Cell;
use ach_util::Error;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Watch<T, const W: usize> {
    val: Cell<T>,
    producer: Notify<W>,
}
impl<T, const W: usize> Watch<T, W> {
    pub const fn new(init: T) -> Self {
        Self {
            val: Cell::new_with(init),
            producer: Notify::new(),
        }
    }
    pub fn try_send(&self, value: T) -> Result<(), Error<T>> {
        self.val.try_replace(value)?;
        self.producer.notify();
        Ok(())
    }
    /// Error if the watch is refering
    /// Notice: Spin
    pub fn send(&self, value: T) -> Result<(), Error<T>> {
        self.val.replace(value)?;
        self.producer.notify();
        Ok(())
    }
    /// Notice: Spin
    pub fn subscribe(&self) -> Receiver<'_, T, W> {
        Receiver {
            parent: self,
            wait_p: self.producer.listen(),
        }
    }
}

pub struct Receiver<'a, T, const W: usize> {
    parent: &'a Watch<T, W>,
    wait_p: Listener<'a, W>,
}
impl<'a, T: Clone, const W: usize> Future for Receiver<'a, T, W> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Pin::new(&mut self.wait_p).poll(cx).is_ready() {
            match self.parent.val.try_get() {
                Ok(val) => Poll::Ready(val.clone()),
                Err(err) if err.retry => Poll::Pending,
                Err(_) => {
                    unreachable!()
                }
            }
        } else {
            Poll::Pending
        }
    }
}
