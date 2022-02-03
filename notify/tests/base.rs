use async_ach_notify::Notify;
use core::future::Future;
use core::pin::Pin;
use futures_test::task;

#[test]
fn test() {
    static NOTIFY: Notify<2> = Notify::new();
    let mut cx = task::noop_context();

    let mut listener1 = NOTIFY.listen();
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_pending());
    NOTIFY.notify_one();
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_pending());

    let mut listener2 = NOTIFY.listen();
    let mut listener3 = NOTIFY.listen();
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut listener2).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut listener3).poll(&mut cx).is_pending());
    NOTIFY.notify_one();
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut listener2).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut listener3).poll(&mut cx).is_pending());
    NOTIFY.notify_one();
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut listener2).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut listener3).poll(&mut cx).is_pending());
    NOTIFY.notify_one();
    assert!(Pin::new(&mut listener1).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut listener2).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut listener3).poll(&mut cx).is_pending());
}
