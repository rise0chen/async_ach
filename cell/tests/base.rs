use async_ach_cell::Cell;
use core::future::Future;
use core::pin::Pin;
use futures_test::task;

#[test]
fn test() {
    static CELL: Cell<usize> = Cell::new();
    let mut cx = task::noop_context();

    let mut get = CELL.get();
    assert!(Pin::new(&mut get).poll(&mut cx).is_ready());

    let mut take = CELL.take();
    assert!(Pin::new(&mut take).poll(&mut cx).is_ready());

    let mut replace = CELL.replace(1);
    assert!(Pin::new(&mut replace).poll(&mut cx).is_ready());

    let mut set = CELL.set(2);
    assert!(Pin::new(&mut set).poll(&mut cx).is_ready());
}
