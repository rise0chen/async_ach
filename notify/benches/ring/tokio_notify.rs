use ach_ring as ach;
use ach_util::Error;
use tokio::sync::Notify;

pub struct Ring<T, const N: usize, const MP: usize, const MC: usize> {
    buf: ach::Ring<T, N>,
    consumer: Notify,
    producer: Notify,
}
impl<T, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
    pub fn new() -> Self {
        Self {
            buf: ach::Ring::new(),
            consumer: Notify::new(),
            producer: Notify::new(),
        }
    }
}
impl<T: Unpin, const N: usize, const MP: usize, const MC: usize> Ring<T, N, MP, MC> {
    pub fn try_push(&self, val: T) -> Result<(), Error<T>> {
        self.buf.push(val).map(|x| {
            self.producer.notify_one();
            x
        })
    }
    pub async fn push(&self, mut val: T) {
        loop {
            let l =self.consumer.notified();
            if let Err(err) = self.try_push(val) {
                val = err.input;
                l.await;
            } else {
                self.consumer.notify_one();
                break;
            }
        }
    }

    pub fn try_pop(&self) -> Result<T, Error<()>> {
        self.buf.pop().map(|x| {
            self.consumer.notify_one();
            x
        })
    }
    pub async fn pop(&self) -> T {
        loop {
            let l=self.producer.notified();
            if let Ok(v) = self.try_pop() {
                self.producer.notify_one();
                break v;
            } else {
                l.await;
            }
        }
    }
}
