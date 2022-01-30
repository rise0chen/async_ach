#![no_std]

use ach_ring as ach;
use async_ach_notify::{Notified, Notify};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Ring<T, const N: usize, const MP: usize, const MC: usize> {
    buf: ach::Ring<T, N>,
    w_p: Notify<MP>,
    w_c: Notify<MC>,
}
impl<T, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
    pub const fn new() -> Self {
        Self {
            buf: ach::Ring::new(),
            w_p: Notify::new(),
            w_c: Notify::new(),
        }
    }
    pub fn pop(&self) -> Pop<T, N, MP, MC> {
        Pop {
            parent: self,
            notified: self.w_c.notified(),
        }
    }
}
impl<T: Unpin, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
    pub fn push(&self, val: T) -> Push<T, N, MP, MC> {
        Push {
            parent: self,
            notified: self.w_p.notified(),
            val: Some(val),
        }
    }
}

pub struct Push<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> {
    parent: &'a Ring<T, N, MP, MC>,
    notified: Notified<'a, MP>,
    val: Option<T>,
}
impl<'a, T: Unpin, const N: usize, const MP: usize, const MC: usize> Future
    for Push<'a, T, N, MP, MC>
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = if let Some(val) = self.val.take() {
            val
        } else {
            return Poll::Ready(());
        };
        if let Err(val) = self.parent.buf.push(val) {
            self.val = Some(val);
            if Pin::new(&mut self.notified).poll(cx).is_ready() {
                self.poll(cx)
            } else {
                Poll::Pending
            }
        } else {
            self.parent.w_c.notify_waiters();
            Poll::Ready(())
        }
    }
}

pub struct Pop<'a, T, const N: usize, const MP: usize, const MC: usize> {
    parent: &'a Ring<T, N, MP, MC>,
    notified: Notified<'a, MC>,
}
impl<'a, T, const N: usize, const MP: usize, const MC: usize> Future for Pop<'a, T, N, MP, MC> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(val) = self.parent.buf.pop() {
            self.parent.w_p.notify_waiters();
            Poll::Ready(val)
        } else {
            if Pin::new(&mut self.notified).poll(cx).is_ready() {
                self.poll(cx)
            } else {
                Poll::Pending
            }
        }
    }
}
