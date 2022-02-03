use async_ach_ring::Ring;
use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use futures_test::task;

#[test]
fn test() {
    static RING: Ring<usize, 2, 2, 2> = Ring::new();
    let mut cx = task::noop_context();

    let mut pop1 = Box::pin(RING.pop());
    assert!(Pin::new(&mut pop1).poll(&mut cx).is_pending());
    let mut push1 = Box::pin(RING.push(1));
    assert!(Pin::new(&mut push1).poll(&mut cx).is_ready());
    assert_eq!(Pin::new(&mut pop1).poll(&mut cx), Poll::Ready(1));

    let mut push2 = Box::pin(RING.push(2));
    assert!(Pin::new(&mut push2).poll(&mut cx).is_ready());
    let mut push3 = Box::pin(RING.push(3));
    assert!(Pin::new(&mut push3).poll(&mut cx).is_ready());
    let mut push4 = Box::pin(RING.push(4));
    assert!(Pin::new(&mut push4).poll(&mut cx).is_pending());
    let mut pop2 = Box::pin(RING.pop());
    assert_eq!(Pin::new(&mut pop2).poll(&mut cx), Poll::Ready(2));
    assert!(Pin::new(&mut push4).poll(&mut cx).is_ready());
}
