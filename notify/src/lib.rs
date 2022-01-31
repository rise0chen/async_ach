#![no_std]

use async_ach_waker::{WakerPool, WakerToken};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use core::task::{Context, Poll};

pub struct Notify<const W: usize> {
    version: AtomicUsize,
    wakers: WakerPool<W>,
}
impl<const W: usize> Notify<W> {
    pub const fn new() -> Self {
        Self {
            version: AtomicUsize::new(0),
            wakers: WakerPool::new(),
        }
    }
    /// Notify all waiters
    pub fn notify(&self) {
        self.version.fetch_add(1, Relaxed);
        self.wakers.wake();
    }
    pub fn listen(&self) -> Listener<'_, W> {
        Listener {
            parent: self,
            version: self.version.load(Relaxed),
            token: None,
        }
    }
}

pub struct Listener<'a, const W: usize> {
    parent: &'a Notify<W>,
    version: usize,
    token: Option<WakerToken<'a, W>>,
}
impl<'a, const W: usize> Future for Listener<'a, W> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let new_version = self.parent.version.load(Relaxed);
        if self.version < new_version {
            self.version = new_version;
            Poll::Ready(())
        } else {
            let waker = cx.waker();
            if let Some(token) = &self.token {
                token.replace(waker);
            } else {
                if let Some(token) = self.parent.wakers.register(waker) {
                    self.token = Some(token);
                } else {
                    // spin
                    waker.wake_by_ref();
                }
            }
            Poll::Pending
        }
    }
}
