use async_ach_watch::Watch;
use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use futures_test::task;

#[test]
fn test() {
    static WATCH: Watch<usize> = Watch::new(0);
    let mut cx = task::noop_context();

    let mut sub1 = WATCH.subscribe();
    let mut get1 = Box::pin(sub1.get());
    assert!(Pin::new(&mut get1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut get1).poll(&mut cx).is_pending());
    WATCH.try_send(1).unwrap();
    assert_eq!(Pin::new(&mut get1).poll(&mut cx), Poll::Ready(1));

    let mut sub2 = WATCH.subscribe();
    let mut sub3 = WATCH.subscribe();
    let mut get2 = Box::pin(sub2.get());
    WATCH.try_send(2).unwrap();
    let mut get3 = Box::pin(sub3.get());
    assert_eq!(Pin::new(&mut get2).poll(&mut cx), Poll::Ready(2));
    assert_eq!(Pin::new(&mut get3).poll(&mut cx), Poll::Ready(2));
}
