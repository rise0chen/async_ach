use async_ach_spsc::Spsc;
use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use futures_test::task;

#[test]
fn test() {
    static SPSC: Spsc<usize, 2> = Spsc::new();
    let mut cx = task::noop_context();

    let mut sender = SPSC.take_sender().unwrap();
    assert!(SPSC.take_sender().is_none());
    let mut recver = SPSC.take_recver().unwrap();
    assert!(SPSC.take_recver().is_none());
    let mut send1 = Box::pin(sender.send(1));
    let mut recv1 = Box::pin(recver.recv());
    assert_eq!(Pin::new(&mut recv1).poll(&mut cx), Poll::Pending);
    assert!(Pin::new(&mut send1).poll(&mut cx).is_ready());
    assert_eq!(Pin::new(&mut recv1).poll(&mut cx), Poll::Ready(1));
}
