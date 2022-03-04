#![no_std]

use ach_util::Error;
use async_ach_cell::Cell;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::task::{Context, Poll};
use core::{ops::Range, time::Duration};
use futures_util::Stream;

pub struct Watch<T, const W: usize> {
    val: Cell<T, 1, 1>,
    version: AtomicUsize,
    producer: Notify<W>,
}
impl<T, const W: usize> Watch<T, W> {
    pub const fn new(init: T) -> Self {
        Self {
            val: Cell::new_with(init),
            version: AtomicUsize::new(0),
            producer: Notify::new(),
        }
    }
}
impl<T: Unpin + Clone, const W: usize> Watch<T, W> {
    /// Get a copy of the value.
    pub fn data(&self) -> T {
        unsafe { self.val.peek().clone() }
    }
    /// Update the watch
    ///
    /// Returns Err if the value in critical section.
    pub fn try_send(&self, value: T) -> Result<(), Error<T>> {
        self.val.try_replace(value)?;
        self.version.fetch_add(1, SeqCst);
        self.producer.notify_waiters();
        Ok(())
    }
    /// Update the watch
    pub async fn send(&self, value: T) {
        self.val.replace(value).await;
        self.version.fetch_add(1, SeqCst);
        self.producer.notify_one();
    }
    pub fn subscribe(&self) -> Receiver<'_, T, W> {
        Receiver {
            parent: self,
            version: self.version.load(SeqCst),
            wait_p: self.producer.listen(),
        }
    }
    fn update(&self, version: &mut usize) -> bool {
        let new_version = self.version.load(SeqCst);
        if *version < new_version {
            *version = new_version;
            true
        } else {
            false
        }
    }
}

pub struct Receiver<'a, T, const W: usize> {
    parent: &'a Watch<T, W>,
    version: usize,
    wait_p: Listener<'a, W>,
}
impl<'a, T: Unpin + Clone, const W: usize> Receiver<'a, T, W> {
    /// Get a copy of the value.
    pub fn data(&self) -> T {
        self.parent.data()
    }
    pub fn changed<'b>(&'b mut self) -> Changed<'b, 'a, T, W> {
        self.changed_interval(Duration::ZERO..Duration::MAX)
    }
    pub fn changed_interval<'b>(&'b mut self, interval: Range<Duration>) -> Changed<'b, 'a, T, W> {
        Changed {
            parent: self,
            interval,
            last_time: 0,
            sleep: None,
        }
    }
}

pub struct Changed<'b, 'a, T, const W: usize> {
    parent: &'b mut Receiver<'a, T, W>,
    interval: Range<Duration>,
    last_time: u64,
    sleep: Option<async_tick::Sleep>,
}
impl<'b, 'a, T: Unpin + Clone, const W: usize> Stream for Changed<'b, 'a, T, W> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let now = async_tick::now();
        if self.interval.start != Duration::ZERO {
            let start = self.last_time + self.interval.start.as_nanos() as u64;
            if now < start {
                let mut sleep = async_tick::sleep_until(start);
                if Pin::new(&mut sleep).poll(cx).is_pending() {
                    self.sleep = Some(sleep);
                    return Poll::Pending;
                }
            }
        }
        if self.interval.end != Duration::MAX {
            let end = self.last_time + self.interval.end.as_nanos() as u64;
            if now > end {
                self.last_time = now;
                return Poll::Ready(Some(self.parent.data()));
            }
        }

        if self.parent.parent.update(&mut self.parent.version) {
            self.last_time = now;
            return Poll::Ready(Some(self.parent.data()));
        }
        if self.interval.end != Duration::MAX {
            let end = self.last_time + self.interval.end.as_nanos() as u64;
            let mut sleep = async_tick::sleep_until(end);
            if Pin::new(&mut sleep).poll(cx).is_ready() {
                self.last_time = now;
                return Poll::Ready(Some(self.parent.data()));
            }
            self.sleep = Some(sleep);
        }
        let _ = Pin::new(&mut self.parent.wait_p).poll_next(cx);
        if self.parent.parent.update(&mut self.parent.version) {
            self.last_time = now;
            Poll::Ready(Some(self.parent.data()))
        } else {
            Poll::Pending
        }
    }
}
impl<'b, 'a, T: Unpin + Clone, const W: usize> Future for Changed<'b, 'a, T, W> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(val) => Poll::Ready(val.unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
}
