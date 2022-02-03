#![no_std]

use ach_util::Error;
use async_ach_cell::Cell;
use async_ach_notify::{Listener, Notify};
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};

pub struct Watch<T, const W: usize> {
    val: Cell<T, 1, W>,
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
impl<T: Unpin, const W: usize> Watch<T, W> {
    /// Update the watch
    ///
    /// Returns Err if the value is refered or in critical section.
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
    pub async fn get(&mut self) -> T {
        loop {
            if self.parent.update(&mut self.version) {
                return self.parent.val.get().await.unwrap().clone();
            } else {
                self.wait_p.clone().await;
            }
        }
    }
}
