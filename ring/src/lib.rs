#![no_std]

use ach_ring as ach;
use ach_util::Error;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

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
}
impl<T: Unpin, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
    /// Appends an element to the back of the Ring.
    ///
    /// Returns Err if the Ring is full or in critical section.
    pub fn try_push(&self, val: T) -> Result<(), Error<T>> {
        self.buf.try_push(val).map(|x| {
            self.producer.notify();
            x
        })
    }
    /// Appends an element to the back of the Ring.
    pub fn push(&self, val: T) -> Push<T, N, MP, MC> {
        Push {
            parent: self,
            wait_c: self.consumer.listen(),
            val: Some(val),
        }
    }

    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty or in critical section.
    pub fn try_pop(&self) -> Result<T, Error<()>> {
        self.buf.try_pop().map(|x| {
            self.consumer.notify();
            x
        })
    }
    /// Removes the first element and returns it.
    pub fn pop(&self) -> Pop<T, N, MP, MC> {
        Pop {
            parent: self,
            wait_p: self.producer.listen(),
        }
    }
}

pub struct Push<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> {
    parent: &'a Ring<T, N, MP, MC>,
    wait_c: Listener<'a, MP>,
    val: Option<T>,
}
impl<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> Future
    for Push<'a, T, N, MP, MC>
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = if let Some(val) = self.val.take() {
            val
        } else {
            return Poll::Ready(());
        };
        match self.parent.try_push(val) {
            Ok(v) => Poll::Ready(v),
            Err(err) => {
                self.val = Some(err.input);
                if Pin::new(&mut self.wait_c).poll(cx).is_ready() {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

pub struct Pop<'a, T, const N: usize, const MP: usize, const MC: usize> {
    parent: &'a Ring<T, N, MP, MC>,
    wait_p: Listener<'a, MC>,
}
impl<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> Future
    for Pop<'a, T, N, MP, MC>
{
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.parent.try_pop() {
            Ok(v) => Poll::Ready(v),
            Err(_) => {
                if Pin::new(&mut self.wait_p).poll(cx).is_ready() {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
