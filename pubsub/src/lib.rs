#![no_std]

use ach_pubsub as ach;
use ach_pubsub::Ref;
use ach_util::Error;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::Stream;

pub struct Subscriber<'a, T, const N: usize, const MP: usize, const MC: usize> {
    parent: &'a Publisher<T, N, MP, MC>,
    ch: Ref<'a, ach::Subscriber<T, N>>,
}
impl<'a, T, const N: usize, const MP: usize, const MC: usize> Subscriber<'a, T, N, MP, MC> {
    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty.
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        let data = self.ch.try_recv()?;
        self.parent.consumer.notify_one();
        Ok(data)
    }
    pub fn recv<'b>(&'b self) -> Recv<'a, 'b, T, N, MP, MC> {
        Recv {
            parent: self,
            wait: self.parent.producer.listen(),
        }
    }
}
pub struct Recv<'a, 'b, T, const N: usize, const MP: usize, const MC: usize> {
    parent: &'b Subscriber<'a, T, N, MP, MC>,
    wait: Listener<'b, MC>,
}
impl<'a, 'b, T, const N: usize, const MP: usize, const MC: usize> Stream
    for Recv<'a, 'b, T, N, MP, MC>
{
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Ok(data) = self.parent.try_recv() {
            Poll::Ready(Some(data))
        } else {
            let _ = Pin::new(&mut self.wait).poll_next(cx);
            if let Ok(data) = self.parent.try_recv() {
                Poll::Ready(Some(data))
            } else {
                Poll::Pending
            }
        }
    }
}
impl<'a, 'b, T, const N: usize, const MP: usize, const MC: usize> Future
    for Recv<'a, 'b, T, N, MP, MC>
{
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(val)) => Poll::Ready(val),
            Poll::Ready(None) => Poll::Pending,
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct Publisher<T, const N: usize, const MP: usize, const MC: usize> {
    ch: ach::Publisher<T, N, MC>,
    consumer: Notify<MP>,
    producer: Notify<MC>,
}
impl<T, const N: usize, const MP: usize, const MC: usize> Publisher<T, N, MP, MC> {
    pub const fn new(strict: bool) -> Self {
        Self {
            ch: ach::Publisher::new(strict),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
    pub fn subscribe(&self) -> Option<Subscriber<T, N, MP, MC>> {
        if let Some(sub) = self.ch.subscribe() {
            Some(Subscriber {
                parent: self,
                ch: sub,
            })
        } else {
            None
        }
    }
}
impl<T: Clone, const N: usize, const MP: usize, const MC: usize> Publisher<T, N, MP, MC> {
    /// return success times
    ///
    /// Notice: `Spin` if strict
    pub fn send(&self, val: T) -> usize {
        let num = self.ch.send(val);
        if num != 0 {
            self.producer.notify_waiters();
        }
        num
    }
}
