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
    pub fn notify_waiters(&self) {
        self.version.fetch_add(1, Relaxed);
        self.wakers.wake();
    }
    /// Notice: Spin
    pub fn notified(&self) -> Notified<'_, W> {
        Notified {
            parent: self,
            version: self.version.load(Relaxed),
            token: None,
        }
    }
}

pub struct Notified<'a, const W: usize> {
    parent: &'a Notify<W>,
    version: usize,
    token: Option<WakerToken<'a, W>>,
}
impl<'a, const W: usize> Future for Notified<'a, W> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let new_version = self.parent.version.load(Relaxed);
        if self.version < new_version {
            self.version = new_version;
            Poll::Ready(())
        } else {
            let waker = cx.waker();
            if let Some(token) = &self.token {
                token.swap(waker);
            } else {
                if let Ok(token) = self.parent.wakers.register(waker) {
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
