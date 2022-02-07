#![no_std]

use ach_cell as ach;
use ach_util::Error;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Ref<'a, T> {
    parent: &'a Cell<T>,
    val: ach::Ref<'a, T>,
}
impl<'a, T> Deref for Ref<'a, T> {
    type Target = ach::Ref<'a, T>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        if self.val.ref_num() == Ok(1) {
            if self.val.will_remove() {
                self.parent.consumer.notify_waiters();
            } else {
                self.parent.producer.notify_waiters();
            }
        }
    }
}

pub struct Cell<T> {
    val: ach::Cell<T>,
    consumer: Notify,
    producer: Notify,
}
impl<T> Cell<T> {
    pub const fn new() -> Self {
        Self {
            val: ach::Cell::new(),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
    pub const fn new_with(val: T) -> Self {
        Self {
            val: ach::Cell::new_with(val),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
}
impl<T: Unpin> Cell<T> {
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized or in critical section.
    pub fn try_get(&self) -> Result<Ref<T>, Error<()>> {
        self.val.try_get().map(|x| Ref {
            parent: self,
            val: x,
        })
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized.
    pub fn get(&self) -> Get<'_, T> {
        Get {
            parent: self,
            wait_p: self.producer.listen(),
        }
    }
    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is refered, initialized or in critical section.
    pub fn try_set(&self, val: T) -> Result<(), Error<T>> {
        self.val.try_set(val).map(|x| {
            self.producer.notify_waiters();
            x
        })
    }
    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is refered or initialized.
    pub fn set(&self, val: T) -> Set<'_, T> {
        Set {
            parent: self,
            wait_c: self.consumer.listen(),
            val: Some(val),
        }
    }
    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err if the cell is refered or in critical section.
    pub fn try_take(&self) -> Result<Option<T>, Error<()>> {
        self.val.try_take().map(|x| {
            self.consumer.notify_waiters();
            x
        })
    }
    /// Takes ownership of the current value, leaving the cell uninitialized.
    pub fn take(&self) -> Take<'_, T> {
        Take {
            parent: self,
            wait_p: self.producer.listen(),
            wait_c: self.consumer.listen(),
        }
    }
    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err if the value is refered or in critical section.
    pub fn try_replace(&self, val: T) -> Result<Option<T>, Error<T>> {
        self.val.try_replace(val).map(|x| {
            self.producer.notify_waiters();
            x
        })
    }
    /// Replaces the contained value with value, and returns the old contained value.
    pub fn replace(&self, val: T) -> Replace<'_, T> {
        Replace {
            parent: self,
            wait_p: self.producer.listen(),
            wait_c: self.consumer.listen(),
            val: Some(val),
        }
    }
}

pub struct Get<'a, T> {
    parent: &'a Cell<T>,
    wait_p: Listener<'a>,
}
impl<'a, T: Unpin> Future for Get<'a, T> {
    type Output = Result<Ref<'a, T>, Error<()>>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.parent.try_get() {
            Ok(v) => Poll::Ready(Ok(v)),
            Err(err) if err.retry => {
                if Pin::new(&mut self.wait_p).poll(cx).is_ready() {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
pub struct Set<'a, T> {
    parent: &'a Cell<T>,
    wait_c: Listener<'a>,
    val: Option<T>,
}
impl<'a, T: Unpin> Future for Set<'a, T> {
    type Output = Result<(), Error<T>>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = self.val.take().expect("resumed after completion");
        match self.parent.try_set(val) {
            Ok(v) => Poll::Ready(Ok(v)),
            Err(err) if err.retry => {
                self.val = Some(err.input);
                if Pin::new(&mut self.wait_c).poll(cx).is_ready() {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

pub struct Take<'a, T> {
    parent: &'a Cell<T>,
    wait_p: Listener<'a>,
    wait_c: Listener<'a>,
}
impl<'a, T: Unpin> Future for Take<'a, T> {
    type Output = Option<T>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.parent.try_take() {
            Ok(v) => Poll::Ready(v),
            Err(_) => {
                if Pin::new(&mut self.wait_p).poll(cx).is_ready()
                    || Pin::new(&mut self.wait_c).poll(cx).is_ready()
                {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
pub struct Replace<'a, T> {
    parent: &'a Cell<T>,
    wait_p: Listener<'a>,
    wait_c: Listener<'a>,
    val: Option<T>,
}
impl<'a, T: Unpin> Future for Replace<'a, T> {
    type Output = Option<T>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = self.val.take().expect("resumed after completion");
        match self.parent.try_replace(val) {
            Ok(v) => Poll::Ready(v),
            Err(err) => {
                self.val = Some(err.input);
                if Pin::new(&mut self.wait_p).poll(cx).is_ready()
                    || Pin::new(&mut self.wait_c).poll(cx).is_ready()
                {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
