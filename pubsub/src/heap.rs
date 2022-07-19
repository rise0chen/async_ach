use ach_pubsub::heap as ach;
use ach_util::Error;
use alloc::sync::Arc;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::Stream;

pub struct Subscriber<T, const N: usize, const MC: usize> {
    ch: ach::Subscriber<T, N, MC>,
    producer: Arc<Notify<MC>>,
}
impl<T, const N: usize, const MC: usize> Subscriber<T, N, MC> {
    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty.
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.ch.try_recv()
    }
    pub fn recv<'b>(&'b self) -> Recv<'b, T, N, MC> {
        Recv {
            parent: self,
            wait: self.producer.listen(),
        }
    }
}
pub struct Recv<'b, T, const N: usize, const MC: usize> {
    parent: &'b Subscriber<T, N, MC>,
    wait: Listener<'b, MC>,
}
impl<'b, T, const N: usize, const MC: usize> Stream for Recv<'b, T, N, MC> {
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
impl<'b, T, const N: usize, const MC: usize> Future for Recv<'b, T, N, MC> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(val)) => Poll::Ready(val),
            Poll::Ready(None) => Poll::Pending,
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct Publisher<T, const N: usize, const MC: usize> {
    ch: ach::Publisher<T, N, MC>,
    producer: Arc<Notify<MC>>,
}
impl<T, const N: usize, const MC: usize> Publisher<T, N, MC> {
    pub fn new(strict: bool) -> Self {
        Self {
            ch: ach::Publisher::new(strict),
            producer: Arc::new(Notify::new()),
        }
    }
    pub fn subscribe(&self) -> Option<Subscriber<T, N, MC>> {
        if let Some(sub) = self.ch.subscribe() {
            Some(Subscriber {
                ch: sub,
                producer: self.producer.clone(),
            })
        } else {
            None
        }
    }
}
impl<T: Clone, const N: usize, const MC: usize> Publisher<T, N, MC> {
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
