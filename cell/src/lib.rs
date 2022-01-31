#![no_std]

use ach_cell as ach;
use ach_util::Error;
use async_ach_notify::{Listener, Notify};
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Ref<'a, T, const MP: usize, const MC: usize> {
    parent: &'a Cell<T, MP, MC>,
    val: ach::Ref<'a, T>,
}
impl<'a, T, const MP: usize, const MC: usize> Deref for Ref<'a, T, MP, MC> {
    type Target = ach::Ref<'a, T>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
impl<'a, T, const MP: usize, const MC: usize> Drop for Ref<'a, T, MP, MC> {
    fn drop(&mut self) {
        if self.val.ref_num() == Ok(1) {
            if self.val.will_remove() {
                self.parent.consumer.notify();
            } else {
                self.parent.producer.notify();
            }
        }
    }
}

pub struct Cell<T, const MP: usize, const MC: usize> {
    val: ach::Cell<T>,
    consumer: Notify<MP>,
    producer: Notify<MC>,
}
impl<T, const MP: usize, const MC: usize> Cell<T, MP, MC> {
    pub const fn new() -> Self {
        Self {
            val: ach::Cell::new(),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
}
impl<T: Unpin, const MP: usize, const MC: usize> Cell<T, MP, MC> {
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized or in critical section.
    pub fn try_get(&self) -> Result<Ref<T, MP, MC>, Error<()>> {
        self.val.try_get().map(|x| Ref {
            parent: self,
            val: x,
        })
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized.
    pub fn get(&self) -> Get<'_, T, MP, MC> {
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
            self.producer.notify();
            x
        })
    }
    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is refered, initialized.
    pub fn set(&self, val: T) -> Set<'_, T, MP, MC> {
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
            self.consumer.notify();
            x
        })
    }
    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err if the cell is refered.
    pub fn take(&self) -> Take<'_, T, MP, MC> {
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
            self.producer.notify();
            x
        })
    }
    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err if the value is refered.
    pub fn replace(&self, val: T) -> Replace<'_, T, MP, MC> {
        Replace {
            parent: self,
            wait_p: self.producer.listen(),
            wait_c: self.consumer.listen(),
            val: Some(val),
        }
    }
}

pub struct Get<'a, T, const MP: usize, const MC: usize> {
    parent: &'a Cell<T, MP, MC>,
    wait_p: Listener<'a, MC>,
}
impl<'a, T: Unpin, const MP: usize, const MC: usize> Future for Get<'a, T, MP, MC> {
    type Output = Result<Ref<'a, T, MP, MC>, Error<()>>;
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
pub struct Set<'a, T, const MP: usize, const MC: usize> {
    parent: &'a Cell<T, MP, MC>,
    wait_c: Listener<'a, MP>,
    val: Option<T>,
}
impl<'a, T: Unpin, const MP: usize, const MC: usize> Future for Set<'a, T, MP, MC> {
    type Output = Result<(), Error<T>>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = if let Some(v) = self.val.take() {
            v
        } else {
            return Poll::Ready(Ok(()));
        };
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

pub struct Take<'a, T, const MP: usize, const MC: usize> {
    parent: &'a Cell<T, MP, MC>,
    wait_p: Listener<'a, MC>,
    wait_c: Listener<'a, MP>,
}
impl<'a, T: Unpin, const MP: usize, const MC: usize> Future for Take<'a, T, MP, MC> {
    type Output = Result<Option<T>, Error<()>>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.parent.try_take() {
            Ok(v) => Poll::Ready(Ok(v)),
            Err(err) if err.retry => {
                if Pin::new(&mut self.wait_p).poll(cx).is_ready()
                    || Pin::new(&mut self.wait_c).poll(cx).is_ready()
                {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
pub struct Replace<'a, T, const MP: usize, const MC: usize> {
    parent: &'a Cell<T, MP, MC>,
    wait_p: Listener<'a, MC>,
    wait_c: Listener<'a, MP>,
    val: Option<T>,
}
impl<'a, T: Unpin, const MP: usize, const MC: usize> Future for Replace<'a, T, MP, MC> {
    type Output = Result<Option<T>, Error<T>>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = if let Some(v) = self.val.take() {
            v
        } else {
            return Poll::Ready(Ok(None));
        };
        match self.parent.try_replace(val) {
            Ok(v) => Poll::Ready(Ok(v)),
            Err(err) if err.retry => {
                self.val = Some(err.input);
                if Pin::new(&mut self.wait_p).poll(cx).is_ready()
                    || Pin::new(&mut self.wait_c).poll(cx).is_ready()
                {
                    self.poll(cx)
                } else {
                    Poll::Pending
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
