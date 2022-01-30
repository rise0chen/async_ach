use async_ach_watch::Watch;
use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use futures_test::task;

#[test]
fn test() {
    static WATCH: Watch<usize, 2> = Watch::new(0);
    let mut cx = task::noop_context();

    let mut sub1 = WATCH.subscribe();
    assert!(Pin::new(&mut sub1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut sub1).poll(&mut cx).is_pending());
    WATCH.send(1).unwrap();
    assert_eq!(Pin::new(&mut sub1).poll(&mut cx), Poll::Ready(1));
    assert!(Pin::new(&mut sub1).poll(&mut cx).is_pending());

    let mut sub2 = WATCH.subscribe();
    let mut sub3 = WATCH.subscribe();
    assert!(Pin::new(&mut sub1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut sub2).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut sub3).poll(&mut cx).is_pending());
    WATCH.send(2).unwrap();
    assert_eq!(Pin::new(&mut sub1).poll(&mut cx), Poll::Ready(2));
    assert_eq!(Pin::new(&mut sub2).poll(&mut cx), Poll::Ready(2));
    assert_eq!(Pin::new(&mut sub3).poll(&mut cx), Poll::Ready(2));
}
