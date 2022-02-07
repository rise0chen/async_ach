#![no_std]

use ach_util::Error;
use async_ach_cell::Cell;
use async_ach_notify::Notify;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};

pub struct Watch<T> {
    val: Cell<T>,
    version: AtomicUsize,
    producer: Notify,
}
impl<T> Watch<T> {
    pub const fn new(init: T) -> Self {
        Self {
            val: Cell::new_with(init),
            version: AtomicUsize::new(0),
            producer: Notify::new(),
        }
    }
}
impl<T: Unpin> Watch<T> {
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
    pub fn subscribe(&self) -> Receiver<'_, T> {
        Receiver {
            parent: self,
            version: self.version.load(SeqCst),
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

pub struct Receiver<'a, T> {
    parent: &'a Watch<T>,
    version: usize,
}
impl<'a, T: Unpin + Clone> Receiver<'a, T> {
    pub async fn get(&mut self) -> T {
        loop {
            if self.parent.update(&mut self.version) {
                return self.parent.val.get().await.unwrap().clone();
            } else {
                self.parent.producer.listen().await;
            }
        }
    }
}
