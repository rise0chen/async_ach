#![no_std]

use ach_spsc as ach;
use async_ach_notify::Notify;

pub struct Spsc<T, const N: usize> {
    buf: ach::Spsc<T, N>,
    consumer: Notify,
    producer: Notify,
}
impl<T, const N: usize> Spsc<T, N> {
    pub const fn new() -> Self {
        Self {
            buf: ach::Spsc::new(),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
}
impl<T: Unpin, const N: usize> Spsc<T, N> {
    pub fn take_sender(&self) -> Option<Sender<T, N>> {
        let sender = self.buf.take_sender()?;
        Some(Sender {
            parent: self,
            sender,
        })
    }
    pub fn take_recver(&self) -> Option<Receiver<T, N>> {
        let recver = self.buf.take_recver()?;
        Some(Receiver {
            parent: self,
            recver,
        })
    }
}

pub struct Sender<'a, T: Unpin, const N: usize> {
    parent: &'a Spsc<T, N>,
    sender: ach::Sender<'a, T, N>,
}
impl<'a, T: Unpin, const N: usize> Sender<'a, T, N> {
    pub fn try_send(&mut self, val: T) -> Result<(), T> {
        self.sender.send(val).map(|_| {
            self.parent.producer.notify_one();
        })
    }
    pub async fn send<'b>(&'b mut self, mut val: T) {
        loop {
            if let Err(v) = self.try_send(val) {
                val = v;
                self.parent.consumer.listen().await;
            } else {
                break;
            }
        }
    }
}

pub struct Receiver<'a, T, const N: usize> {
    parent: &'a Spsc<T, N>,
    recver: ach::Receiver<'a, T, N>,
}
impl<'a, T: Unpin, const N: usize> Receiver<'a, T, N> {
    pub fn try_recv(&mut self) -> Option<T> {
        self.recver.recv().map(|v| {
            self.parent.consumer.notify_one();
            v
        })
    }
    pub async fn recv<'b>(&'b mut self) -> T {
        loop {
            if let Some(v) = self.try_recv() {
                break v;
            } else {
                self.parent.producer.listen().await;
            }
        }
    }
}
