use crate::heapless::Mpmc;
use ach_util::Error;
use alloc::sync::Arc;

#[derive(Clone)]
pub struct Sender<T: Unpin, const N: usize, const MP: usize, const MC: usize> {
    tx: Arc<Mpmc<T, N, MP, MC>>,
}
impl<T: Unpin, const N: usize, const MP: usize, const MC: usize> Sender<T, N, MP, MC> {
    pub fn try_send(&self, val: T) -> Result<(), Error<T>> {
        self.tx.sender().try_send(val)
    }
    pub async fn send(&self, val: T) {
        self.tx.sender().send(val).await
    }
}

#[derive(Clone)]
pub struct Receiver<T: Unpin, const N: usize, const MP: usize, const MC: usize> {
    rx: Arc<Mpmc<T, N, MP, MC>>,
}
impl<T: Unpin, const N: usize, const MP: usize, const MC: usize> Receiver<T, N, MP, MC> {
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.rx.recver().try_recv()
    }
    pub async fn recv(&self) -> T {
        self.rx.recver().recv().await
    }
}

pub fn channel<T: Unpin, const N: usize, const MP: usize, const MC: usize>(
) -> (Sender<T, N, MP, MC>, Receiver<T, N, MP, MC>) {
    let tx = Arc::new(Mpmc::new());
    let rx = tx.clone();
    (Sender { tx }, Receiver { rx })
}
