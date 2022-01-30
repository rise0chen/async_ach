use async_ach_notify::Notify;
use core::future::Future;
use core::pin::Pin;
use futures_test::task;

#[test]
fn test() {
    static NOTIFY: Notify<2> = Notify::new();
    let mut cx = task::noop_context();

    let mut notified1 = NOTIFY.notified();
    assert!(Pin::new(&mut notified1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut notified1).poll(&mut cx).is_pending());
    NOTIFY.notify_waiters();
    assert!(Pin::new(&mut notified1).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut notified1).poll(&mut cx).is_pending());

    let mut notified2 = NOTIFY.notified();
    let mut notified3 = NOTIFY.notified();
    assert!(Pin::new(&mut notified1).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut notified2).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut notified3).poll(&mut cx).is_pending());
    NOTIFY.notify_waiters();
    assert!(Pin::new(&mut notified1).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut notified2).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut notified3).poll(&mut cx).is_ready());
}
