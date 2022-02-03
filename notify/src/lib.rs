#![no_std]

use async_ach_waker::WakerPool;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::task::{Context, Poll};

pub struct Notify<const W: usize> {
    permit: AtomicUsize,
    wakers: WakerPool<W>,
}
impl<const W: usize> Notify<W> {
    pub const fn new() -> Self {
        Self {
            permit: AtomicUsize::new(0),
            wakers: WakerPool::new(),
        }
    }
    /// Notify a waiter
    pub fn notify_one(&self) {
        self.permit.fetch_add(1, SeqCst);
        self.wakers.wake_one();
    }
    pub fn notify_waiters(&self) {
        loop {
            self.permit.fetch_add(1, SeqCst);
            if !self.wakers.wake_one() {
                self.get_permit();
                break;
            }
        }
    }
    /// Had been notified
    pub fn had_notified(&self) -> bool {
        self.permit.load(SeqCst) != 0
    }
    /// Wait for a notice
    pub fn listen(&self) -> Listener<'_, W> {
        Listener { parent: self }
    }
    fn get_permit(&self) -> bool {
        self.permit
            .fetch_update(SeqCst, SeqCst, |x| if x > 0 { Some(x - 1) } else { None })
            .is_ok()
    }
}

#[derive(Clone)]
pub struct Listener<'a, const W: usize> {
    parent: &'a Notify<W>,
}
impl<'a, const W: usize> Future for Listener<'a, W> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.parent.get_permit() {
            Poll::Ready(())
        } else {
            let waker = cx.waker();
            if !self.parent.wakers.register(waker) {
                // spin
                waker.wake_by_ref();
            }
            if self.parent.get_permit() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}
