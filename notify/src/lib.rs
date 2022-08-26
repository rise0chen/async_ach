#![no_std]

use async_ach_waker::pool::{WakerPool, WakerToken};
use async_ach_waker::WakerEntity;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::task::{Context, Poll};
use futures_util::Stream;

pub struct Notify<const W: usize> {
    permit: AtomicUsize,
    wakers: WakerPool<(), W>,
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
    pub fn notify_waiters(&self) -> usize {
        let mut num = 0;
        loop {
            self.permit.fetch_add(1, SeqCst);
            if !self.wakers.wake_one() {
                self.get_permit();
                break;
            } else {
                num += 1;
            }
        }
        num
    }
    /// Had been notified
    pub fn had_notified(&self) -> bool {
        self.permit.load(SeqCst) != 0
    }
    /// Wait for a notice
    pub fn listen(&self) -> Listener<'_, W> {
        Listener {
            parent: self,
            token: None,
        }
    }
    fn get_permit(&self) -> bool {
        self.permit
            .fetch_update(SeqCst, SeqCst, |x| if x > 0 { Some(x - 1) } else { None })
            .is_ok()
    }
}

pub struct Listener<'a, const W: usize> {
    parent: &'a Notify<W>,
    token: Option<WakerToken<'a, (), W>>,
}
impl<'a, const W: usize> Listener<'a, W> {
    pub fn pendable(&mut self) -> bool {
        if let Some(_) = &self.token {
            true
        } else if let Ok(token) = self.parent.wakers.register() {
            self.token = Some(token);
            true
        } else {
            false
        }
    }
}
impl<'a, const W: usize> Stream for Listener<'a, W> {
    type Item = ();
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let waker = cx.waker();
        if self.pendable() {
            self.token
                .as_ref()
                .unwrap()
                .swap(WakerEntity::new(waker.clone(), ()));
        } else {
            waker.wake_by_ref();
        }
        if self.parent.get_permit() {
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}
impl<'a, const W: usize> Future for Listener<'a, W> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}
