use crate::heapless::Spsc;
use alloc::sync::Arc;

pub struct Sender<T: Unpin, const N: usize> {
    tx: Arc<Spsc<T, N>>,
}
impl<T: Unpin, const N: usize> Sender<T, N> {
    pub fn try_send(&mut self, val: T) -> Result<(), T> {
        self.tx.take_sender().unwrap().try_send(val)
    }
    pub async fn send(&mut self, val: T) {
        self.tx.take_sender().unwrap().send(val).await
    }
}

pub struct Receiver<T: Unpin, const N: usize> {
    rx: Arc<Spsc<T, N>>,
}
impl<T: Unpin, const N: usize> Receiver<T, N> {
    pub fn try_recv(&mut self) -> Option<T> {
        self.rx.take_recver().unwrap().try_recv()
    }
    pub async fn recv(&mut self) -> T {
        self.rx.take_recver().unwrap().recv().await
    }
}

pub fn channel<T: Unpin, const N: usize>() -> (Sender<T, N>, Receiver<T, N>) {
    let tx = Arc::new(Spsc::new());
    let rx = tx.clone();
    (Sender { tx }, Receiver { rx })
}
