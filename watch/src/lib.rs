#![no_std]

use ach_cell::Cell;
use async_ach_notify::{Notified, Notify};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Watch<T, const W: usize> {
    val: Cell<T>,
    notify: Notify<W>,
}
impl<T, const W: usize> Watch<T, W> {
    pub const fn new(init: T) -> Self {
        Self {
            val: Cell::new_with(init),
            notify: Notify::new(),
        }
    }
    /// Error if the watch is refering
    /// Notice: Spin
    pub fn send(&self, value: T) -> Result<(), T> {
        self.val.swap(value)?;
        self.notify.notify_waiters();
        Ok(())
    }
    /// Notice: Spin
    pub fn subscribe(&self) -> Receiver<'_, T, W> {
        Receiver {
            parent: self,
            notified: self.notify.notified(),
        }
    }
}

pub struct Receiver<'a, T, const W: usize> {
    parent: &'a Watch<T, W>,
    notified: Notified<'a, W>,
}
impl<'a, T: Clone, const W: usize> Future for Receiver<'a, T, W> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Pin::new(&mut self.notified).poll(cx).is_ready() {
            let output = if let Some(v) = self.parent.val.get() {
                v.clone()
            } else {
                unreachable!()
            };
            Poll::Ready(output)
        } else {
            Poll::Pending
        }
    }
}
