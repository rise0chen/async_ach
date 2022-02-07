#![no_std]

use async_ach_waker::linked::{WakerLinked, WakerNode};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::task::{Context, Poll};

pub struct Notify {
    permit: AtomicUsize,
    wakers: WakerLinked,
}
impl Notify {
    pub const fn new() -> Self {
        Self {
            permit: AtomicUsize::new(0),
            wakers: WakerLinked::new(),
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
    pub fn listen(&self) -> Listener {
        Listener {
            parent: self,
            waker: WakerNode::new(),
        }
    }
    fn get_permit(&self) -> bool {
        self.permit
            .fetch_update(SeqCst, SeqCst, |x| if x > 0 { Some(x - 1) } else { None })
            .is_ok()
    }
}

pub struct Listener<'a> {
    parent: &'a Notify,
    waker: WakerNode,
}
impl<'a> Future for Listener<'a> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.parent.get_permit() {
            Poll::Ready(())
        } else {
            self.waker.register(cx.waker());
            unsafe {
                self.parent
                    .wakers
                    .push(&mut *(&self.waker as *const _ as *mut _))
            };
            if self.parent.get_permit() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}
